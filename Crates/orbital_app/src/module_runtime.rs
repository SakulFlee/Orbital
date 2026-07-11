//! Module-based runtime — replaces AppRuntime for ECS-native applications.
//!
//! Uses the `Module` trait instead of `App`. Multiple modules contribute
//! systems to a shared game schedule. The runtime manages core, game, and
//! realize schedules.

use std::sync::{Arc, Mutex};

#[cfg(feature = "gamepad_input")]
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
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, WindowId},
};

use crate::{make_core_schedule, AppContext, AppSettings, AppState, Module, Timer};
use orbital_ecs_bridge::{
    CursorPosition, DeltaTime, DeviceResource, EngineEvent, EngineEvents, FrameCounter,
    InputSnapshot, QueueResource, TotalTime, WindowSize,
};

macro_rules! ctx_lock {
    ($ctx:ident) => {
        $ctx.lock().expect("Mutex failure")
    };
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
    realize_schedule: Schedule,
    module_setup_done: bool,
    #[cfg(feature = "gamepad_input")]
    gil: Gilrs,
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
            realize_schedule: Schedule::new(),
            module_setup_done: false,
            #[cfg(feature = "gamepad_input")]
            gil: Gilrs::new().expect("Gamepad input initialization failed!"),
        };

        // Initialise built-in ECS resources
        runtime.ecs_world.insert_resource(FrameCounter(0));
        runtime.ecs_world.insert_resource(DeltaTime(0.0));
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

        event_loop.run_app(&mut runtime)
    }

    fn redraw(&mut self) {
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

        let view: wgpu::TextureView = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(format),
            ..TextureViewDescriptor::default()
        });

        // Run realize schedule (dirty → GPU sync)
        self.realize_schedule.run(&mut self.ecs_world);

        // TODO: Build bind group from ECS, render
        // For now, this is a placeholder — will be wired up in Phase 5e

        lock.queue().present(frame);
    }

    #[cfg(feature = "gamepad_input")]
    fn receive_controller_inputs(&mut self) {
        while let Some(gil_event) = self.gil.next_event() {
            if let Some(input_event) = InputEvent::convert_gil_event(gil_event) {
                self.input_state.handle_event(input_event);
            }
        }
    }

    fn update(&mut self) -> bool {
        let (delta_time, cycle) = self.timer.as_mut().expect("Timer went missing").tick();

        if let Some((total_delta, fps)) = cycle {
            info!("FPS: {fps} | TDT: {total_delta}s | CDT: {delta_time}s");
        }

        // Write frame-computed engine state into the ECS world
        self.ecs_world.insert_resource(DeltaTime(delta_time));
        self.ecs_world
            .insert_resource(InputSnapshot(self.input_state.clone()));

        // Run core schedule (timing, frame counter)
        self.core_schedule.run(&mut self.ecs_world);

        // Run game schedule (user systems)
        self.game_schedule.run(&mut self.ecs_world);

        #[cfg(feature = "gamepad_input_poll")]
        self.receive_controller_inputs();

        // Process engine events
        let exit_requested = self.process_engine_events();

        self.input_state.reset_deltas();

        exit_requested
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
                    let lock = ctx_lock!(ctx);
                    if grab {
                        if let Err(e) = lock.window().set_cursor_grab(CursorGrabMode::Confined) {
                            error!(
                                "Failed to set cursor grab! This might not be supported on your platform. Error: {e}"
                            );
                        }
                    } else if let Err(e) = lock.window().set_cursor_grab(CursorGrabMode::None) {
                        error!("Failed to unset cursor grab! Error: {e}");
                    }
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
        event_loop.exit();
    }
}

impl ApplicationHandler for ModuleRuntime {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !matches!(self.state, AppState::Starting) || !matches!(self.state, AppState::Paused) {
            debug!(
                "Attempting to resume while not in required state! (State: {:?})",
                self.state
            );
        }

        debug!("Resuming app ...");

        let ctx = match AppContext::new(event_loop, &self.settings) {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("Critical error: Failed to acquire context while resuming app!");
                trace!("Error: {:?}", e);
                return;
            }
        };

        let _config = ctx.make_surface_configuration(self.settings.vsync_enabled);

        self.state = AppState::Ready(Arc::new(Mutex::new(ctx)));

        self.timer = Some(Timer::new());

        if let AppState::Ready(ctx_arc) = &self.state {
            let ctx_guard = ctx_lock!(ctx_arc);

            self.ecs_world
                .insert_resource(DeviceResource(Arc::new(ctx_guard.device().clone())));
            self.ecs_world
                .insert_resource(QueueResource(Arc::new(ctx_guard.queue().clone())));

            // Call Module::setup() and build game schedule
            if !self.module_setup_done {
                let systems = self.module.setup(
                    &mut self.ecs_world,
                    ctx_guard.device(),
                    ctx_guard.queue(),
                );
                for system in systems {
                    self.game_schedule.add_system_boxed(system);
                }
                self.module_setup_done = true;
                info!("Module setup complete, game schedule has {} systems", self.game_schedule.system_count());
            }
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

        self.state = AppState::Paused;
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
            } => Some(InputEvent::MouseWheel {
                device_id,
                delta,
                phase,
            }),
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
                let ctx_lock = ctx_lock!(ctx);
                let configuration =
                    ctx_lock.make_surface_configuration(self.settings.vsync_enabled);
                ctx_lock.reconfigure_surface(&configuration);

                self.input_state.surface_resize(new_size);
                self.ecs_world
                    .insert_resource(WindowSize(cgmath::Vector2::new(new_size.width, new_size.height)));

                None
            }
            _ => None,
        };

        if let Some(input_event) = input_event {
            self.input_state.handle_event(input_event);
        }
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
