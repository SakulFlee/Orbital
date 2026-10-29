//! Module-based runtime — replaces AppRuntime for ECS-native applications.
//!
//! Uses the `Module` trait instead of `App`. Multiple modules contribute
//! systems to a shared game schedule. The runtime manages core, game, and
//! realize schedules.

use std::sync::{Arc, Mutex};

#[cfg(all(
    feature = "gamepad_gilrs",
    not(any(target_os = "android", target_os = "ios"))
))]
use gilrs::Gilrs;
use log::trace;
use orbital_core::logging::{self, debug, error, info, warn};
use orbital_ecs::Schedule;
use orbital_input::{InputEvent, InputState};
use wgpu::CurrentSurfaceTexture;
use wgpu::TextureViewDescriptor;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{
        DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{CursorGrabMode, WindowId},
};

use crate::{AppContext, AppSettings, AppState, Module, Timer, make_core_schedule};
use orbital_ecs_bridge::{
    ActiveCamera, AdapterResource, CameraDescriptor, CameraDirty, CameraStore, CursorGrabConfig,
    CursorGrabState, CursorPosition, DeltaTime, DeviceResource, EngineEvent, EngineEvents,
    FpsStats, FrameCounter, FrameTimings, IcedCapturedMouseDrag, IcedCapturedMouseWheel,
    IcedCapturedTouches, IcedEventQueue, IcedTextInputFocus, InputSnapshot, QueueResource,
    STAGE_BIND_MODELS, STAGE_CULL_EXTRACT, STAGE_GAME_SCHEDULE, STAGE_GPU_MODELS,
    STAGE_GPU_SCENE_TOTAL, STAGE_GPU_SHADOW, STAGE_GPU_SKYBOX, STAGE_ICED_DRAW, STAGE_ICED_SUBMIT,
    STAGE_OVERLAY_LAYERS, STAGE_OVERLAY_MAIN, STAGE_PRESENT, STAGE_REALIZE, STAGE_REALIZE_ENV,
    STAGE_REALIZE_MODELS, STAGE_SCENE_ENCODE, STAGE_SCENE_SUBMIT, STAGE_STAGGER, STAGE_SURFACE_ACQ,
    STAGE_TIMESTAMP_READBACK, STAGE_TOTAL, STAGE_UI_EVENTS, STAGE_UPDATE, SurfaceFormatResource,
    TIMING_STAGES, TimingsHistoryConfig, TotalTime, WindowSize,
};

macro_rules! ctx_lock {
    ($ctx:ident) => {
        $ctx.lock().expect("Mutex failure")
    };
}

/// Physically apply a cursor grab to a window.
///
/// "Not supported" is expected on platforms without pointer constraints
/// (Android/iOS) and is downgraded to a debug log so routine focus
/// transitions don't spam errors; genuine grab failures still log a warning.
fn apply_cursor_grab(window: &winit::window::Window, mode: CursorGrabMode) {
    match window.set_cursor_grab(mode) {
        Ok(()) => {}
        Err(e @ winit::error::ExternalError::NotSupported(_)) => {
            debug!("Cursor grab ({mode:?}) is not supported on this platform: {e}");
        }
        Err(e) => warn!("Cursor grab ({mode:?}) failed: {e}"),
    }
}

/// Scratch buffer for the render frame's stage measurements.
///
/// Filled once per frame and handed to [`FrameTimings::record`], so recording
/// does not allocate. The stage order must match [`TIMING_STAGES`].
struct FrameStageScratch {
    durations: Vec<f64>,
}

impl FrameStageScratch {
    fn new() -> Self {
        Self {
            durations: vec![0.0; TIMING_STAGES.len()],
        }
    }

    fn set(&mut self, stage: usize, duration: std::time::Duration) {
        self.durations[stage] = duration.as_secs_f64() * 1000.0;
    }

    fn set_ms(&mut self, stage: usize, milliseconds: f64) {
        self.durations[stage] = milliseconds;
    }

    fn set_gpu(&mut self, gpu_ns: [f64; 4]) {
        // gpu_ns: [shadow_start, skybox_start, models_start, main_end].
        const NS_TO_MS: f64 = 1.0 / 1_000_000.0;

        self.durations[STAGE_GPU_SHADOW] = (gpu_ns[1] - gpu_ns[0]) * NS_TO_MS;
        self.durations[STAGE_GPU_SKYBOX] = (gpu_ns[2] - gpu_ns[1]) * NS_TO_MS;
        self.durations[STAGE_GPU_MODELS] = (gpu_ns[3] - gpu_ns[2]) * NS_TO_MS;
        self.durations[STAGE_GPU_SCENE_TOTAL] = (gpu_ns[3] - gpu_ns[0]) * NS_TO_MS;
    }
}

/// CPU timings captured during `update()`, before the redraw that records
/// them. `update()` runs immediately before `redraw()` in the same event.
#[derive(Debug, Clone, Copy, Default)]
struct UpdateTimings {
    total_ms: f64,
    ui_events_ms: f64,
    game_schedule_ms: f64,
}

/// Cumulative creation counters sampled once per timing-log window.
#[derive(Debug, Clone, Copy, Default)]
struct PerfCounterSnapshot {
    shader_modules: u64,
    unique_wgsl: u64,
    pipelines: u64,
    material_hits: u64,
    material_misses: u64,
    sky_updates: u64,
}

/// A wheel event held back from game input until iced has processed it.
///
/// Winit reports wheel deltas as Copy values, so this keeps the event alive
/// without borrowing the original `WindowEvent`.
struct DeferredWheel {
    device_id: DeviceId,
    delta: MouseScrollDelta,
    phase: TouchPhase,
}

pub struct ModuleRuntime {
    module: Box<dyn Module>,
    settings: AppSettings,
    state: AppState,
    input_state: InputState,
    timer: Option<Timer>,
    ecs_world: orbital_ecs::World,
    core_schedule: Schedule,
    game_schedule: Schedule,
    module_setup_done: bool,
    renderer: Option<orbital_renderer::Renderer>,
    frame_timings: FrameStageScratch,
    update_timings: UpdateTimings,
    perf_counters: PerfCounterSnapshot,
    back_press_count: u8,
    last_back_press: Option<std::time::Instant>,
    /// Touch events deferred for game-input processing.
    ///
    /// During `window_event`, touches are pushed to the iced event queue
    /// immediately but game-input processing is deferred to `update()`.
    /// This allows `IcedLayerRenderer::process_events()` to run first and
    /// populate `IcedCapturedTouches`, so the game knows which touches the
    /// UI consumed before deciding whether to forward them.
    deferred_touches: Vec<winit::event::Touch>,
    /// Wheel events deferred for game-input processing.
    ///
    /// Like touches, a wheel is pushed to the iced queue immediately but its
    /// game-input handling waits for `update()`, so `process_events()` can
    /// decide whether a panel captured it first.
    deferred_wheels: Vec<DeferredWheel>,
    /// Whether the mouse pointer is currently over the window surface.
    ///
    /// Used to defer `set_cursor_grab` calls: on Wayland, requesting a grab
    /// with no pointer over the surface leaves winit's grab bookkeeping out
    /// of sync and can make it issue a second `confine_pointer` without
    /// releasing the previous one — a fatal `already_constrained` protocol
    /// error. The grab is applied on the next `CursorEntered` instead.
    pointer_over_window: bool,
    /// Whether the window currently has keyboard focus.
    window_focused: bool,
    /// Desired grab state for apps that set [`CursorGrabConfig`] without a
    /// [`CursorGrabState`] resource. When the resource exists it takes
    /// precedence, so direct writes to it are still honoured.
    cursor_grab_desired: bool,
    /// Overlay renderers, owned by the runtime (not the ECS world) so that
    /// `update()` can call `process_events()` with `&mut World` without
    /// borrow-checker conflicts.
    overlay_renderers: std::sync::Mutex<Vec<Box<dyn crate::render_overlay::LayerRenderer>>>,
    #[cfg(all(
        feature = "gamepad_gilrs",
        not(any(target_os = "android", target_os = "ios"))
    ))]
    /// Gamepad event source. `None` when gilrs fails to initialize (e.g. no
    /// input subsystem) — the app runs without gamepad support instead of
    /// panicking at startup.
    gil: Option<Gilrs>,
    #[cfg(all(feature = "gamepad_input_poll", target_os = "ios"))]
    ios_gamepad: crate::ios_gamepad::IosGamepad,
}

impl ModuleRuntime {
    pub fn liftoff<M: Module + 'static>(
        event_loop: EventLoop<()>,
        settings: AppSettings,
        module: M,
    ) -> Result<(), EventLoopError> {
        logging::init();

        info!("Orbital Runtime (Module)");
        info!(" --- @SakulFlee --- ");

        let mut runtime = Self {
            module: Box::new(module),
            settings,
            state: AppState::Starting,
            input_state: InputState::new(),
            timer: None,
            ecs_world: orbital_ecs::World::new(),
            core_schedule: make_core_schedule(),
            game_schedule: Schedule::new(),
            module_setup_done: false,
            renderer: None,
            frame_timings: FrameStageScratch::new(),
            update_timings: UpdateTimings::default(),
            perf_counters: PerfCounterSnapshot::default(),
            back_press_count: 0,
            last_back_press: None,
            deferred_touches: Vec::new(),
            deferred_wheels: Vec::new(),
            pointer_over_window: false,
            window_focused: false,
            cursor_grab_desired: false,
            overlay_renderers: std::sync::Mutex::new(Vec::new()),
            #[cfg(all(
                feature = "gamepad_gilrs",
                not(any(target_os = "android", target_os = "ios"))
            ))]
            gil: match Gilrs::new() {
                Ok(gil) => Some(gil),
                Err(e) => {
                    warn!("Gamepad input unavailable, continuing without: {e}");
                    None
                }
            },
            #[cfg(all(feature = "gamepad_input_poll", target_os = "ios"))]
            ios_gamepad: crate::ios_gamepad::IosGamepad::new(),
        };

        // Initialise built-in ECS resources
        runtime.ecs_world.insert_resource(FrameCounter(0));
        runtime.ecs_world.insert_resource(DeltaTime(0.0));
        runtime.ecs_world.insert_resource(FpsStats::default());
        runtime.ecs_world.insert_resource(FrameTimings::default());
        runtime.ecs_world.insert_resource(TotalTime(0.0));
        runtime
            .ecs_world
            .insert_resource(InputSnapshot(InputState::new()));
        runtime
            .ecs_world
            .insert_resource(CursorPosition(cgmath::Vector2::new(0.0, 0.0)));
        runtime
            .ecs_world
            .insert_resource(WindowSize(cgmath::Vector2::new(0, 0)));
        runtime.ecs_world.insert_resource(EngineEvents::default());
        runtime.ecs_world.insert_resource(CameraStore::new());

        event_loop.run_app(&mut runtime)
    }

    fn redraw(&mut self) {
        let t_start = std::time::Instant::now();

        let AppState::Ready(ctx) = &self.state else {
            error!(
                "Trying to redraw when app state is in a non-ready state! ({:?})",
                self.state
            );
            return;
        };

        let lock = ctx_lock!(ctx);

        let frame = match lock.current_surface_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                warn!("Suboptimal surface texture acquired!");
                surface_texture
            }
            CurrentSurfaceTexture::Timeout => {
                warn!("Acquiring next surface texture encountered a timeout!");
                return;
            }
            CurrentSurfaceTexture::Occluded => {
                warn!("Cannot acquire next surface texture as the surface is current occluded!");
                return;
            }
            CurrentSurfaceTexture::Outdated => {
                warn!("Acquired next surface texture, but its already outdated!");
                return;
            }
            CurrentSurfaceTexture::Lost => {
                warn!("Surface was lost, cannot acquire next surface texture!");
                return;
            }
            CurrentSurfaceTexture::Validation => {
                warn!("Encountered a validation error while acquiring surface texture");
                return;
            }
        };

        let format = lock.get_first_view_format();
        let device = lock.device();
        let queue = lock.queue();

        let view: wgpu::TextureView = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(format),
            ..TextureViewDescriptor::default()
        });

        let _t_ecs_start = std::time::Instant::now();

        // ─── section timing ──────────────────────────────────────────
        use std::time::Instant;
        let t_realize_start = Instant::now();

        // Check for any model modifications BEFORE realize clears ModelDirty.
        // This catches newly-imported models (e.g. DamagedHelmet) whose
        // ModelDirty(true) is cleared by realize_models on the same frame;
        // if we checked AFTER realize, global_model_dirty would always be
        // false and shadow maps would never include the new model.
        let global_model_dirty = {
            if let Some(store) = self
                .ecs_world
                .get_component_store::<orbital_ecs_bridge::ModelDirty>()
            {
                store.dense.iter().any(|&eid| {
                    store
                        .sparse
                        .get(eid)
                        .copied()
                        .flatten()
                        .map(|idx| store.components[idx].0)
                        .unwrap_or(false)
                })
            } else {
                false
            }
        };

        // Realize all ECS state (needs &mut self.ecs_world)
        crate::systems::realize::realize_cameras(&mut self.ecs_world);
        crate::systems::realize::realize_lights(&mut self.ecs_world);
        let t_env_start = Instant::now();
        crate::systems::realize::realize_environment(&mut self.ecs_world);
        let t_models_start = Instant::now();
        crate::systems::realize::realize_models(&mut self.ecs_world);
        let t_stagger_start = Instant::now();
        let d_realize = t_stagger_start - t_realize_start;
        let d_realize_env = t_models_start - t_env_start;
        let d_realize_models = t_stagger_start - t_models_start;

        // Check if new lights were added this frame (forces full bootstrap)
        let new_light_bootstrap = self
            .ecs_world
            .get_resource::<orbital_ecs_bridge::NewLightBootstrap>()
            .map(|n| n.0)
            .unwrap_or(false);
        if new_light_bootstrap {
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::NewLightBootstrap(false));
        }

        // Schedule shadow updates respecting the per-frame budget
        let dirty_set = crate::systems::stagger::sys_stagger_shadow_updates(
            &mut self.ecs_world,
            global_model_dirty,
            new_light_bootstrap,
        );

        log::trace!(
            "stagger: gmd={} nlb={} dirty_set_len={}",
            global_model_dirty,
            new_light_bootstrap,
            dirty_set.len(),
        );
        let t_cull_start = Instant::now();
        let d_stagger = t_cull_start - t_stagger_start;

        // Freeze/unfreeze the culling frustum (default F4)
        {
            use orbital_input::InputButton;
            use std::sync::atomic::Ordering;
            use winit::keyboard::PhysicalKey;

            static PREV_FREEZE: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);

            let freeze_key = self
                .ecs_world
                .get_resource::<crate::FreezeKeyConfig>()
                .map(|c| c.0)
                .unwrap_or(winit::keyboard::KeyCode::F4);

            let input_res = self.ecs_world.get_resource::<InputSnapshot>();
            let pressed = input_res
                .map(|input| {
                    input
                        .0
                        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(freeze_key)))
                        .map(|(_, s)| s)
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if pressed && !PREV_FREEZE.load(Ordering::Relaxed) {
                // Rising edge — toggle frozen state.
                // Read the current state (immutable borrow of ecs_world).
                let was_frozen = self
                    .ecs_world
                    .get_resource::<orbital_ecs_bridge::FrozenFrustum>()
                    .and_then(|f| f.0.clone())
                    .is_some();

                // Read camera data while ecs_world is still immutably borrowed.
                let capture = if !was_frozen {
                    let active_cam = self
                        .ecs_world
                        .get_resource::<orbital_ecs_bridge::ActiveCamera>();
                    let store = self
                        .ecs_world
                        .get_resource::<orbital_ecs_bridge::CameraStore>();
                    active_cam.and_then(|active| {
                        store.and_then(|s| {
                            s.get(active.0.index).map(|arc_cam| {
                                let cam = arc_cam.read().unwrap();
                                orbital_ecs_bridge::FrozenFrustumData {
                                    frustum: cam.frustum(),
                                    perspective_view_projection_matrix: *cam
                                        .perspective_view_projection_matrix(),
                                }
                            })
                        })
                    })
                } else {
                    None
                };

                // Now mutate ecs_world.
                self.ecs_world
                    .insert_resource(orbital_ecs_bridge::FrozenFrustum(if was_frozen {
                        None
                    } else {
                        capture
                    }));
            }
            PREV_FREEZE.store(pressed, Ordering::Relaxed);
        }

        // CPU frustum culling — tests bounding spheres against the camera
        // frustum and compacts visible instances into a vertex buffer.
        crate::systems::cull::sys_frustum_cull(&mut self.ecs_world);

        // Keep a borrow of the cull resource for the render call below.
        // This borrows self.ecs_world immutably — OK because all subsequent
        // accesses of ecs_world are also immutable.
        let cull_res = self
            .ecs_world
            .get_resource::<orbital_ecs_bridge::CullResource>();

        // Snapshot all renderer-facing ECS state. `RenderScene` owns cheap
        // `Arc`/wgpu clones, so the ECS is no longer borrowed during render.
        let t_bind_start = Instant::now();
        let scene = orbital_renderer::RenderScene::from_world(&self.ecs_world);
        let t_render_start = Instant::now();
        let d_cull_extract = t_bind_start - t_cull_start;
        let d_bind_group_models = t_render_start - t_bind_start;

        // The camera buffer is needed while recording main-encoder layers, so
        // clone the cheap handle before borrowing the renderer mutably.
        let camera_buffer = self
            .renderer
            .as_ref()
            .map(|renderer| renderer.camera_buffer(&scene).clone());
        let screen_size = if let Some(size_res) = self.ecs_world.get_resource::<WindowSize>() {
            (size_res.0.x as f32, size_res.0.y as f32)
        } else {
            (800.0, 600.0)
        };

        let mut layer_renderers = self.overlay_renderers.lock().unwrap();
        layer_renderers.sort_by_key(|r| r.layer().index());

        let d_overlay_main = std::cell::Cell::new(std::time::Duration::ZERO);
        if let (Some(renderer), Some(camera_buffer)) = (&mut self.renderer, camera_buffer.as_ref())
        {
            let cull = cull_res.as_ref().and_then(|r| r.0.as_ref());
            let ecs = &self.ecs_world;
            let mut overlay_callback = |encoder: &mut wgpu::CommandEncoder| {
                let start = Instant::now();
                for layer in layer_renderers.iter_mut() {
                    if layer.submission() != crate::OverlaySubmission::MainEncoder {
                        continue;
                    }
                    let ctx = crate::RenderOverlayContext {
                        target_view: &view,
                        camera_buffer,
                        device,
                        queue,
                        encoder: Some(encoder),
                        ecs,
                        screen_size,
                    };
                    layer.render(ctx);
                }
                d_overlay_main.set(start.elapsed());
            };

            renderer.render(
                &view,
                &scene,
                device,
                queue,
                cull,
                &dirty_set,
                Some(&mut overlay_callback),
            );
            drop(overlay_callback);
        }
        drop(cull_res);
        let d_overlay_main = d_overlay_main.get();

        let renderer_cpu = self
            .renderer
            .as_ref()
            .map(|renderer| renderer.prev_cpu_timings())
            .unwrap_or_default();
        let gpu_ns = self
            .renderer
            .as_ref()
            .map_or([0.0; 4], |renderer| renderer.prev_gpu_ns());

        // Owned-submission layers (iced) always run after the main submission.
        let mut d_overlay_layers = std::time::Duration::ZERO;
        let mut d_iced_draw = std::time::Duration::ZERO;
        let mut d_iced_submit = std::time::Duration::ZERO;
        if let Some(camera_buffer) = camera_buffer.as_ref() {
            let t_layers_start = Instant::now();
            for layer in layer_renderers.iter_mut() {
                if layer.submission() != crate::OverlaySubmission::Owned {
                    continue;
                }
                let ctx = crate::RenderOverlayContext {
                    target_view: &view,
                    camera_buffer,
                    device,
                    queue,
                    encoder: None,
                    ecs: &self.ecs_world,
                    screen_size,
                };
                layer.render(ctx);
                let timings = layer.take_render_timings();
                d_iced_draw += timings.draw;
                d_iced_submit += timings.submit;
            }
            d_overlay_layers = t_layers_start.elapsed();

            // Drain iced events once after all layer renderers have processed.
            // Each IcedLayerRenderer clones (not drains) the queue so that
            // multiple panels all receive the same events.
            if let Some(mut queue) = self.ecs_world.get_resource_mut::<IcedEventQueue>() {
                queue.drain();
            }
        }
        drop(layer_renderers);

        let t_present_start = Instant::now();
        lock.queue().present(frame);
        let d_present = t_present_start.elapsed();
        drop(lock);

        let d_total = Instant::now() - t_realize_start;
        let d_surface_acq = t_realize_start - t_start;

        #[cfg(target_os = "android")]
        {
            let cpu_work = d_total.saturating_sub(d_present);
            crate::android_performance::report_frame(cpu_work);
        }

        self.frame_timings.set(STAGE_SURFACE_ACQ, d_surface_acq);
        self.frame_timings
            .set_ms(STAGE_UPDATE, self.update_timings.total_ms);
        self.frame_timings
            .set_ms(STAGE_UI_EVENTS, self.update_timings.ui_events_ms);
        self.frame_timings
            .set_ms(STAGE_GAME_SCHEDULE, self.update_timings.game_schedule_ms);
        self.frame_timings.set(STAGE_REALIZE, d_realize);
        self.frame_timings
            .set(STAGE_REALIZE_MODELS, d_realize_models);
        self.frame_timings.set(STAGE_REALIZE_ENV, d_realize_env);
        self.frame_timings.set(STAGE_STAGGER, d_stagger);
        self.frame_timings.set(STAGE_CULL_EXTRACT, d_cull_extract);
        self.frame_timings
            .set(STAGE_BIND_MODELS, d_bind_group_models);
        self.frame_timings
            .set_ms(STAGE_SCENE_ENCODE, renderer_cpu.encode_ms);
        self.frame_timings
            .set_ms(STAGE_SCENE_SUBMIT, renderer_cpu.submit_ms);
        self.frame_timings
            .set_ms(STAGE_TIMESTAMP_READBACK, renderer_cpu.timestamp_readback_ms);
        self.frame_timings.set(STAGE_OVERLAY_MAIN, d_overlay_main);
        self.frame_timings
            .set(STAGE_OVERLAY_LAYERS, d_overlay_layers);
        self.frame_timings.set(STAGE_ICED_DRAW, d_iced_draw);
        self.frame_timings.set(STAGE_ICED_SUBMIT, d_iced_submit);
        self.frame_timings.set(STAGE_PRESENT, d_present);
        self.frame_timings.set(STAGE_TOTAL, d_total);
        self.frame_timings.set_gpu(gpu_ns);

        // Publish the timings so debug overlays and systems can read them
        // without going through the log.
        let history = self
            .ecs_world
            .get_resource::<TimingsHistoryConfig>()
            .map(|config| *config)
            .unwrap_or_default();
        let elapsed = self
            .ecs_world
            .get_resource::<TotalTime>()
            .map(|time| time.0)
            .unwrap_or(0.0);
        let cdt_ms = self
            .ecs_world
            .get_resource::<DeltaTime>()
            .map(|delta| delta.0 * 1000.0)
            .unwrap_or(0.0);

        let logged = if let Some(mut timings) = self.ecs_world.get_resource_mut::<FrameTimings>() {
            timings.record_at(&self.frame_timings.durations, elapsed, cdt_ms, history);
            timings.try_log().is_some()
        } else {
            false
        };

        if logged {
            self.log_perf_counters();
        }
    }

    /// Logs creation/cache counters for the same one-second window as
    /// [`FrameTimings::try_log`].
    fn log_perf_counters(&mut self) {
        let shader = orbital_shader_core::stats();
        let (material_hits, material_misses) = orbital_model::material_cache_stats();
        let current = PerfCounterSnapshot {
            shader_modules: shader.modules_created,
            unique_wgsl: shader.unique_sources,
            pipelines: orbital_material_shader::pipelines_created(),
            material_hits,
            material_misses,
            sky_updates: orbital_world_environment::dynamic_sky_updates(),
        };

        info!(
            "PERF window: shader_modules=+{} pipelines=+{} unique_wgsl={} material_cache hits=+{} misses=+{} sky_updates=+{}",
            current
                .shader_modules
                .saturating_sub(self.perf_counters.shader_modules),
            current
                .pipelines
                .saturating_sub(self.perf_counters.pipelines),
            current.unique_wgsl,
            current
                .material_hits
                .saturating_sub(self.perf_counters.material_hits),
            current
                .material_misses
                .saturating_sub(self.perf_counters.material_misses),
            current
                .sky_updates
                .saturating_sub(self.perf_counters.sky_updates),
        );

        self.perf_counters = current;
    }

    #[cfg(all(
        feature = "gamepad_gilrs",
        not(any(target_os = "android", target_os = "ios"))
    ))]
    fn receive_controller_inputs(&mut self) {
        let Some(gil) = self.gil.as_mut() else {
            return;
        };
        while let Some(gil_event) = gil.next_event() {
            if let Some(input_event) = InputEvent::convert_gil_event(gil_event) {
                self.input_state.handle_event(input_event);
            }
        }
    }

    fn update(&mut self) -> bool {
        let update_start = std::time::Instant::now();
        let (delta_time, cycle) = self.timer.as_mut().expect("Timer went missing").tick();

        if let Some((total_delta, fps)) = cycle {
            info!("FPS: {fps} | TDT: {total_delta}s | CDT: {delta_time}s");
            self.ecs_world.insert_resource(FpsStats {
                fps,
                total_delta_time: total_delta,
                cycle_delta_time: delta_time,
            });
        } else {
            // Update cycle_delta_time every frame so the HUD shows the
            // latest per-frame delta even between one-second boundaries.
            if let Some(mut stats) = self.ecs_world.get_resource_mut::<FpsStats>() {
                stats.cycle_delta_time = delta_time;
            }
        }

        // Write frame-computed engine state into the ECS world
        self.ecs_world.insert_resource(DeltaTime(delta_time));

        // ── Process iced UI events BEFORE game systems ──────────────
        // IcedLayerRenderer::process_events() clones the queued events,
        // runs UserInterface::update(), and populates IcedCapturedTouches.
        // This must happen before game touch input so the capture state
        // is current when we decide which touches reach the game.
        if let Some(mut captured) = self.ecs_world.get_resource_mut::<IcedCapturedMouseWheel>() {
            // Wheel capture is per-event, not a persistent gesture.
            captured.0 = false;
        }
        if let Some(mut focus) = self.ecs_world.get_resource_mut::<IcedTextInputFocus>() {
            // Focus is re-probed from every panel below.
            focus.0 = false;
        }

        let t_ui_events_start = std::time::Instant::now();
        {
            let mut layer_renderers = self.overlay_renderers.lock().unwrap();
            for renderer in layer_renderers.iter_mut() {
                renderer.process_events(&mut self.ecs_world);
            }
        }
        self.update_timings.ui_events_ms = t_ui_events_start.elapsed().as_secs_f64() * 1000.0;

        // ── Process deferred game touch input ───────────────────────
        // Touches were deferred in window_event() so that iced could
        // declare captures first.  Now process them using the current
        // IcedCapturedTouches state.
        let deferred = std::mem::take(&mut self.deferred_touches);
        for touch in deferred {
            // If iced captured this touch, skip game-input processing.
            if let Some(captured) = self.ecs_world.get_resource::<IcedCapturedTouches>()
                && captured.contains(touch.id)
            {
                let game_phases =
                    touch.phase == TouchPhase::Started || touch.phase == TouchPhase::Moved;
                if game_phases && self.input_state.touch_position(touch.id).is_some() {
                    // Touch leaked through before capture was known —
                    // synthesise a Cancelled so the game releases the finger.
                    info!(
                        "iced: touch {} captured by UI — cancelling game input",
                        touch.id
                    );
                    self.input_state.handle_event(InputEvent::Touch {
                        device_id: touch.device_id,
                        phase: TouchPhase::Cancelled,
                        location: touch.location,
                        id: touch.id,
                        force: touch.force,
                    });
                }
                if game_phases {
                    // Drop this event for the game.  The iced queue already
                    // has every touch so UI widgets keep working.
                    continue;
                }
                // Ended/Cancelled forwarded to clear game-input state.
            }

            self.input_state.handle_event(InputEvent::Touch {
                device_id: touch.device_id,
                phase: touch.phase,
                location: touch.location,
                id: touch.id,
                force: touch.force,
            });
        }

        // ── Process deferred game wheel input ───────────────────────
        // Like touches, wheels were deferred so iced could claim them first.
        // The flag is a union across panels: if any panel captured a wheel
        // this update, none of the deferred wheels reach the game.
        let wheel_captured = self
            .ecs_world
            .get_resource::<IcedCapturedMouseWheel>()
            .is_some_and(|captured| captured.0);
        let deferred_wheels = std::mem::take(&mut self.deferred_wheels);

        if !wheel_captured {
            for wheel in deferred_wheels {
                self.input_state.handle_event(InputEvent::MouseWheel {
                    device_id: wheel.device_id,
                    delta: wheel.delta,
                    phase: wheel.phase,
                });
            }
        }

        #[cfg(all(
            feature = "gamepad_input_poll",
            not(any(target_os = "android", target_os = "ios"))
        ))]
        // Poll gamepads BEFORE taking the input snapshot and running the
        // schedules, so this frame's stick/button input is visible to the
        // game systems instead of arriving one frame late.
        self.receive_controller_inputs();

        // iOS pads arrive through the GameController provider (winit has
        // no gamepad path there); drain them BEFORE the snapshot so game
        // systems see this frame's state, like desktop's gilrs poll.
        #[cfg(all(feature = "gamepad_input_poll", target_os = "ios"))]
        for input_event in self.ios_gamepad.poll() {
            self.input_state.handle_event(input_event);
        }

        // A focused debug text field owns all game input. Clear the live input
        // state after all event sources have been polled so typing cannot also
        // move the player or camera.
        let text_focus = self
            .ecs_world
            .get_resource::<IcedTextInputFocus>()
            .is_some_and(|focus| focus.0);
        if text_focus {
            self.input_state.clear();
        }

        // Snapshot input state AFTER deferred touches are processed so game
        // systems (camera controller, etc.) see the current frame's touch
        // data — including right-side look deltas.
        // `take_snapshot` moves the per-frame delta maps into the resource
        // instead of deep-cloning the whole `InputState`.
        self.ecs_world
            .insert_resource(InputSnapshot(self.input_state.take_snapshot()));

        // Run core schedule (timing, frame counter)
        self.core_schedule.run(&mut self.ecs_world);

        // Poll importer for completed glTF imports
        crate::systems::sys_poll_importer(&mut self.ecs_world);

        // Run game schedule (user systems). Pass the accumulated game time
        // so systems marked with a desired interval (low priority) are
        // throttled against TotalTime instead of running every frame.
        let now_secs = self
            .ecs_world
            .get_resource::<TotalTime>()
            .map(|t| t.0)
            .unwrap_or(0.0);
        let t_game_schedule_start = std::time::Instant::now();
        self.game_schedule
            .run_with_time(&mut self.ecs_world, now_secs);
        self.update_timings.game_schedule_ms =
            t_game_schedule_start.elapsed().as_secs_f64() * 1000.0;

        // Process engine events
        let exit_requested = self.process_engine_events();

        self.input_state.reset_deltas();
        self.update_timings.total_ms = update_start.elapsed().as_secs_f64() * 1000.0;

        exit_requested
    }

    /// Desired grab state, preferring the ECS resource when present so direct
    /// writes to [`CursorGrabState`] are honoured.
    fn desired_cursor_grab(&self) -> bool {
        self.ecs_world
            .get_resource::<CursorGrabState>()
            .map(|state| state.0)
            .unwrap_or(self.cursor_grab_desired)
    }

    /// Apply the desired cursor grab if the pointer is over the window.
    ///
    /// Wayland's pointer-constraints protocol allows only one constraint per
    /// surface, and winit's Wayland backend can desynchronise its grab
    /// bookkeeping when a grab is requested while no pointer is over the
    /// surface. Defer until `CursorEntered` in that case.
    fn sync_cursor_grab(&mut self, ctx: &Arc<Mutex<AppContext>>) {
        if !self.pointer_over_window || !self.window_focused {
            return;
        }

        let grabbed = self.desired_cursor_grab();
        let mode = if grabbed {
            CursorGrabMode::Confined
        } else {
            CursorGrabMode::None
        };

        let lock = ctx_lock!(ctx);
        apply_cursor_grab(lock.window(), mode);
        lock.window().set_cursor_visible(!grabbed);
    }

    fn process_engine_events(&mut self) -> bool {
        let ctx = match &self.state {
            AppState::Ready(ctx) => ctx.clone(),
            _ => {
                debug!(
                    "App in invalid state ({:?}), skipping engine events!",
                    self.state
                );
                return false;
            }
        };

        let events = match self.ecs_world.get_resource_mut::<EngineEvents>() {
            Some(mut e) => e.drain(),
            None => return false,
        };

        let mut exit_requested = false;
        for event in events {
            match event {
                EngineEvent::CursorGrabbed(grab) => {
                    self.cursor_grab_desired = grab;
                    self.sync_cursor_grab(&ctx);
                }
                EngineEvent::CursorVisible(visible) => {
                    ctx_lock!(ctx).window().set_cursor_visible(visible);
                }
                EngineEvent::RequestClose => {
                    warn!("App closure was requested!");
                    exit_requested = true;
                }
                EngineEvent::ForceClose { exit_code } => {
                    warn!("Force app closure was requested with exit code {exit_code}!");
                    std::process::exit(exit_code);
                }
                EngineEvent::RequestRedraw => {
                    ctx_lock!(ctx).window().request_redraw();
                }
            }
        }

        exit_requested
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        // Clean exit: drop any persisted state so the next launch is fresh.
        self.module.clear_state(&mut self.ecs_world);
        event_loop.exit();
    }
}

impl ApplicationHandler for ModuleRuntime {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Starting | AppState::Paused(_)) {
            debug!(
                "Attempting to resume while not in required state! (State: {:?})",
                self.state
            );
        }

        debug!("Resuming app ...");

        // Reuse the existing context (window/device/queue) when resuming from
        // a pause, so we don't create a second window or rebuild the GPU
        // device. The surface (dropped on suspend) is recreated below. Only
        // build a fresh context on the first start.
        let (ctx, config) = match std::mem::replace(&mut self.state, AppState::Starting) {
            AppState::Paused(ctx) => {
                // Resume: the surface was dropped on suspend, so rebuild it
                // against the (recreated) native window. Creating a second
                // surface from the same window crashes on Android, which is why
                // this only happens here and not on first start.
                let config = {
                    let mut ctx_guard = ctx_lock!(ctx);
                    ctx_guard.recreate_surface(self.settings.vsync_enabled)
                };
                (ctx, config)
            }
            _ => match AppContext::new(event_loop, &self.settings) {
                Ok(ctx) => {
                    let config = ctx.make_surface_configuration(self.settings.vsync_enabled);
                    (Arc::new(Mutex::new(ctx)), config)
                }
                Err(e) => {
                    error!("Critical error: Failed to acquire context while resuming app!");
                    trace!("Error: {:?}", e);
                    self.state = AppState::Starting;
                    return;
                }
            },
        };

        self.state = AppState::Ready(ctx);

        self.timer = Some(Timer::new());

        if let AppState::Ready(ctx_arc) = &self.state {
            let ctx_guard = ctx_lock!(ctx_arc);

            self.ecs_world
                .insert_resource(DeviceResource(Arc::new(ctx_guard.device().clone())));
            self.ecs_world
                .insert_resource(QueueResource(Arc::new(ctx_guard.queue().clone())));
            self.ecs_world
                .insert_resource(AdapterResource(Arc::new(ctx_guard.adapter().clone())));
            self.ecs_world
                .insert_resource(SurfaceFormatResource(config.format));
            self.ecs_world.insert_resource(IcedEventQueue::default());
            self.ecs_world
                .insert_resource(IcedCapturedTouches::default());
            self.ecs_world
                .insert_resource(IcedCapturedMouseDrag::default());
            self.ecs_world
                .insert_resource(IcedCapturedMouseWheel::default());
            self.ecs_world
                .insert_resource(IcedTextInputFocus::default());

            // Initialize import pipeline resources
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::ImportQueueResource::default());
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::ImporterResource::new(4));
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::MeshCacheResource::default());
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::MaterialCacheResource::default());
            self.ecs_world
                .insert_resource(orbital_ecs_bridge::PipelineCacheResource::default());

            // Create renderer
            self.renderer = Some(orbital_renderer::Renderer::new(
                config.format,
                cgmath::Vector2::new(config.width, config.height),
                ctx_guard.device(),
                ctx_guard.queue(),
            ));

            // Apply the app's scene anti-aliasing mode.
            if let Some(renderer) = &mut self.renderer {
                renderer.set_anti_aliasing(self.settings.anti_aliasing);
                renderer.set_smaa_quality(self.settings.smaa_quality);
            }

            // Enable shadows with default settings
            if let Some(renderer) = &mut self.renderer {
                renderer.enable_shadows(
                    ctx_guard.device(),
                    ctx_guard.queue(),
                    16,   // max shadow slots
                    2048, // shadow map resolution
                );
            }

            // Call Module::setup() and build game schedule
            if !self.module_setup_done {
                // Register all shader node libraries with the global registry
                // before any shader assembly occurs.
                orbital_shader_preprocessor::register_global_library(
                    &orbital_shader_math::math_library(),
                )
                .expect("failed to register math library");
                orbital_shader_preprocessor::register_global_library(
                    &orbital_shader_engine::engine_library(),
                )
                .expect("failed to register engine library");
                orbital_shader_preprocessor::register_global_library(
                    &orbital_shader_pbr::pbr_library(),
                )
                .expect("failed to register PBR library");
                orbital_shader_preprocessor::register_global_library(
                    &orbital_world_environment::world_environment_library(),
                )
                .expect("failed to register world-environment library");

                let systems =
                    self.module
                        .setup(&mut self.ecs_world, ctx_guard.device(), ctx_guard.queue());
                for system in systems {
                    self.game_schedule.add_system_boxed(system);
                }

                // Let the module register overlay renderers with the runtime.
                {
                    let mut layer_renderers = self.overlay_renderers.lock().unwrap();
                    self.module
                        .register_overlays(&mut self.ecs_world, &mut layer_renderers);
                }

                // Register engine-level systems
                // (sys_poll_importer is called directly in update() since it takes &mut World)

                self.module_setup_done = true;
                info!(
                    "Module setup complete, game schedule has {} systems",
                    self.game_schedule.system_count()
                );

                // Restore any persisted state (only runs once on a fresh
                // process start, since setup is guarded by module_setup_done).
                self.module.restore_state(&mut self.ecs_world);
            }

            // Auto-grab cursor if configured. Only apply it when the pointer
            // is already over the surface; otherwise `CursorEntered` /
            // `Focused(true)` re-applies it once winit can create a valid
            // pointer constraint (see `sync_cursor_grab`).
            if let Some(config) = self.ecs_world.get_resource::<CursorGrabConfig>()
                && config.0
            {
                self.cursor_grab_desired = true;
                if self.pointer_over_window {
                    apply_cursor_grab(ctx_guard.window(), CursorGrabMode::Confined);
                    ctx_guard.window().set_cursor_visible(false);
                }
            }

            // Seed the input surface size immediately: mouse-delta
            // normalization (and touch-gesture zones) depend on
            // `InputState::surface_size`, which otherwise is only set by a
            // WindowEvent::Resized. If the platform never delivers an
            // initial resize, mouse deltas would arrive as raw, unnormalized
            // pixel counts (wildly wrong sensitivity).
            if config.width > 0 && config.height > 0 {
                self.input_state
                    .surface_resize(winit::dpi::PhysicalSize::new(config.width, config.height));
            }

            // After a suspend→resume (e.g. Android rotation), the surface is
            // recreated with the new native-window dimensions but the camera's
            // aspect ratio still holds the old value.  Sync it now so the
            // projection matches the fresh surface.
            let new_aspect = config.width as f32 / config.height as f32;
            if let Some(active_camera) = self.ecs_world.get_resource::<ActiveCamera>() {
                let eid = active_camera.0.index;
                if let Some(desc_store) =
                    self.ecs_world.get_component_store_mut::<CameraDescriptor>()
                    && let Some(idx) = desc_store.sparse[eid]
                {
                    desc_store.get_mut_store().components[idx].aspect = new_aspect;
                }
                if let Some(dirty_store) = self.ecs_world.get_component_store_mut::<CameraDirty>()
                    && let Some(idx) = dirty_store.sparse[eid]
                {
                    dirty_store.get_mut_store().components[idx].0 = true;
                }
            }
            self.ecs_world
                .insert_resource(WindowSize(cgmath::Vector2::new(
                    config.width,
                    config.height,
                )));
        }

        info!("App resumed.");
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Ready { .. }) {
            debug!(
                "Attempting to suspend while not in ready state! (State: {:?})",
                self.state
            );
        }

        self.module.save_state(&mut self.ecs_world);

        // Move the context into `Paused` (keeping the window/device/queue
        // alive) but drop the GPU surface and renderer, which reference the
        // native window that Android destroys on suspend. They are recreated
        // on resume.
        self.state = match std::mem::replace(&mut self.state, AppState::Starting) {
            AppState::Ready(ctx) => {
                ctx_lock!(ctx).drop_surface();
                self.renderer = None;
                AppState::Paused(ctx)
            }
            other => other,
        };
        info!("App suspended!");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let ctx = match &self.state {
            AppState::Ready(ctx) => ctx.clone(),
            _ => {
                debug!(
                    "App in invalid state ({:?}), skipping window events!",
                    self.state
                );
                return;
            }
        };

        // Android Back: always forwarded to the input system so apps can use
        // single back actions. Additionally, when configured, N consecutive
        // presses within `back_exit_window` quit the app.
        if self.settings.back_presses_to_exit > 0
            && let WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } = &event
            && event.logical_key == Key::Named(NamedKey::BrowserBack)
            && event.state == ElementState::Pressed
            && !event.repeat
        {
            let now = std::time::Instant::now();
            let within_window = self
                .last_back_press
                .map(|t| now.duration_since(t) <= self.settings.back_exit_window)
                .unwrap_or(false);
            self.back_press_count = if within_window {
                self.back_press_count.saturating_add(1)
            } else {
                1
            };
            self.last_back_press = Some(now);

            if self.back_press_count >= self.settings.back_presses_to_exit {
                info!(
                    "Back pressed {} times within window; exiting",
                    self.back_press_count
                );
                self.exit(event_loop);
                return;
            }
        }

        // Forward relevant events to iced UI overlays
        if let Some(mut queue) = self.ecs_world.get_resource_mut::<IcedEventQueue>() {
            use orbital_ecs_bridge::IcedWindowEvent;

            // Update scale factor from window
            let scale = ctx_lock!(ctx).window().scale_factor();
            queue.set_scale_factor(scale);

            match &event {
                WindowEvent::CursorMoved { position, .. } => {
                    queue.push(IcedWindowEvent::CursorMoved {
                        position: *position,
                    });
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    queue.push(IcedWindowEvent::MouseInput {
                        state: *state,
                        button: *button,
                    });
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    queue.push(IcedWindowEvent::MouseWheel { delta: *delta });
                }
                WindowEvent::KeyboardInput {
                    event,
                    is_synthetic,
                    ..
                } => {
                    queue.push(IcedWindowEvent::KeyboardInput {
                        event: event.clone(),
                        is_synthetic: *is_synthetic,
                    });
                }
                WindowEvent::ModifiersChanged(mods) => {
                    queue.push(IcedWindowEvent::ModifiersChanged(mods.state()));
                }
                WindowEvent::Resized(size) => {
                    queue.push(IcedWindowEvent::Resized(*size));
                }
                WindowEvent::Focused(focused) => {
                    queue.push(IcedWindowEvent::Focused(*focused));
                }
                WindowEvent::Touch(touch) => {
                    // Touch-only platforms (Android) emit no synthetic mouse
                    // events, so forward these for iced hit-testing.
                    queue.push(IcedWindowEvent::Touch(*touch));
                }
                WindowEvent::RedrawRequested => {
                    // Not forwarded: a winit-level signal, not iced input.
                    // Forwarding it forces a useless interface.update() with a
                    // Window(RedrawRequested) event on every frame per panel.
                }
                _ => {}
            }
        }

        // A released primary button ends any UI-captured look-drag before
        // the next process_events() run.
        if let WindowEvent::MouseInput {
            state: ElementState::Released,
            button: MouseButton::Left,
            ..
        } = &event
            && let Some(mut captured) = self.ecs_world.get_resource_mut::<IcedCapturedMouseDrag>()
        {
            captured.release();
        }

        // Keep the OS-level cursor grab in sync with focus, while treating
        // `CursorGrabState` as the *desired* state (owned by CursorToggle/Alt)
        // rather than a snapshot of the OS grab:
        //   focus lost  -> release the grab + show the cursor so the
        //                  compositor doesn't keep pointer constraints
        //                  active (the Wayland "cursor visible but iced UI
        //                  unresponsive" issue), but leave the desired state
        //                  untouched;
        //   focus gained -> re-apply the desired grab. Without this, mouse
        //                  look stays dead after any alt-tab or an
        //                  unfocused startup: on Wayland mouse-motion device
        //                  events only flow while a pointer constraint is
        //                  active, and nothing else re-acquired it.
        //
        // Re-application is deferred until the pointer is over the surface
        // (`CursorEntered`) if it left while the window was unfocused.
        match event {
            WindowEvent::Focused(false) => {
                self.window_focused = false;
                let lock = ctx_lock!(ctx);
                apply_cursor_grab(lock.window(), CursorGrabMode::None);
                lock.window().set_cursor_visible(true);
                // A missed button release must not leave look-drag
                // suppressed forever.
                if let Some(mut captured) =
                    self.ecs_world.get_resource_mut::<IcedCapturedMouseDrag>()
                {
                    captured.release();
                }
            }
            WindowEvent::Focused(true) => {
                self.window_focused = true;
                if self.desired_cursor_grab() {
                    self.sync_cursor_grab(&ctx);
                }
            }
            _ => {}
        }

        let input_event = match event {
            WindowEvent::CloseRequested => {
                info!("App shutdown requested!");
                self.exit(event_loop);
                return;
            }
            WindowEvent::RedrawRequested => {
                if self.update() {
                    info!("App shutdown requested!");
                    self.exit(event_loop);
                    return;
                }
                self.redraw();

                #[cfg(feature = "auto_request_redraw")]
                ctx_lock!(ctx).window().request_redraw();

                None
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => Some(InputEvent::KeyboardButton {
                device_id,
                event,
                is_synthetic,
            }),
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => Some(InputEvent::MouseButton {
                device_id,
                state,
                button,
            }),
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                // Deferred until iced has decided whether a panel captured it.
                self.deferred_wheels.push(DeferredWheel {
                    device_id,
                    delta,
                    phase,
                });
                None
            }
            WindowEvent::Touch(touch) => {
                // Game-input processing is deferred to update() — see
                // `deferred_touches`.  By the time those touches are
                // processed, IcedLayerRenderer::process_events() will have
                // populated IcedCapturedTouches so we know which touches
                // the UI consumed.
                self.deferred_touches.push(touch);
                None
            }
            WindowEvent::CursorEntered { .. } => {
                self.pointer_over_window = true;
                self.sync_cursor_grab(&ctx);
                None
            }
            WindowEvent::CursorLeft { .. } => {
                // Keep the desired grab state; re-apply on the next enter.
                self.pointer_over_window = false;
                None
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Some(mut pos) = self.ecs_world.get_resource_mut::<CursorPosition>() {
                    pos.0 = cgmath::Vector2::new(position.x, position.y);
                }
                Some(InputEvent::MouseMovedPosition {
                    device_id,
                    position,
                })
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width == 0 || new_size.height == 0 {
                    return;
                }

                let ctx_lock = ctx_lock!(ctx);
                let configuration =
                    ctx_lock.make_surface_configuration(self.settings.vsync_enabled);
                ctx_lock.reconfigure_surface(&configuration);

                self.input_state.surface_resize(new_size);
                self.ecs_world
                    .insert_resource(WindowSize(cgmath::Vector2::new(
                        new_size.width,
                        new_size.height,
                    )));

                // Recreate the depth texture to match the new surface dimensions.
                let resolution = cgmath::Vector2::new(new_size.width, new_size.height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.change_resolution(resolution, ctx_lock.device(), ctx_lock.queue());
                }

                // Update camera aspect ratio so the projection isn't stretched.
                let new_aspect = new_size.width as f32 / new_size.height as f32;
                if let Some(active_camera) = self.ecs_world.get_resource::<ActiveCamera>() {
                    let eid = active_camera.0.index;
                    if let Some(desc_store) =
                        self.ecs_world.get_component_store_mut::<CameraDescriptor>()
                        && let Some(idx) = desc_store.sparse[eid]
                    {
                        desc_store.get_mut_store().components[idx].aspect = new_aspect;
                    }
                    if let Some(dirty_store) =
                        self.ecs_world.get_component_store_mut::<CameraDirty>()
                        && let Some(idx) = dirty_store.sparse[eid]
                    {
                        dirty_store.get_mut_store().components[idx].0 = true;
                    }
                }

                None
            }
            _ => None,
        };

        if let Some(input_event) = input_event {
            self.input_state.handle_event(input_event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        device_event: DeviceEvent,
    ) {
        if !matches!(&self.state, AppState::Ready(_)) {
            debug!(
                "App in invalid state ({:?}), skipping device events!",
                self.state
            );
            return;
        }

        if let Some(input_event) = InputEvent::convert_device_event(device_id, device_event) {
            self.input_state.handle_event(input_event);
        }
    }
}
