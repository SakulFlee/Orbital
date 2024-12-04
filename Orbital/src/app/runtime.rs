use std::sync::Arc;

use async_std::channel::{Receiver, Sender};
use cgmath::Vector2;
use futures::FutureExt;
use gilrs::Gilrs;
use log::{debug, error, info, warn};

use wgpu::{
    util::{backend_bits_from_env, dx12_shader_compiler_from_env, gles_minor_version_from_env},
    Adapter, Backend, Backends, CompositeAlphaMode, Device, DeviceDescriptor, DeviceType, Features,
    Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryHints, PresentMode, Queue, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureUsages, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{self, DeviceEvent, DeviceId, WindowEvent},
    event_loop::{self, ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    error::Error,
    input::{InputEvent, InputState},
    timer::Timer,
};

use super::{App, AppChange, AppEvent, AppSettings};

pub struct AppRuntime {
    // Events
    event_tx: Sender<AppEvent>,
    app_change_rx: Receiver<AppChange>,
    // App related
    runtime_settings: AppSettings,
    // Window related
    window: Option<Arc<Window>>,
    surface: Option<Surface<'static>>,
    surface_configuration: Option<SurfaceConfiguration>,
    // Device related
    instance: Option<Instance>,
    adapter: Option<Adapter>,
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    timer: Option<Timer>,
    /// Indicates whether we already have a frame acquired that is currently
    /// "in flight" to prevent WGPU from crashing if we re-request the next
    /// frame while the previous one isn't presented yet.
    frame_acquired: bool,
    input_state: InputState,
    #[cfg(feature = "gamepad_input")]
    gil: Gilrs,
}

pub static mut WINDOW_HALF_SIZE: (i32, i32) = (0, 0);

impl AppRuntime {
    pub fn liftoff(
        event_loop: EventLoop<()>,
        settings: AppSettings,
        mut app: impl App + Send + 'static,
    ) -> Result<(), Error> {
        info!("Orbital Runtime");
        info!(" --- @SakulFlee --- ");

        let (event_tx, event_rx) = async_std::channel::unbounded::<AppEvent>();
        let (app_change_tx, app_change_rx) = async_std::channel::unbounded::<AppChange>();

        let mut app_runtime = Self {
            event_tx: event_tx.clone(),
            app_change_rx,
            runtime_settings: settings,
            window: None,
            surface: None,
            surface_configuration: None,
            instance: None,
            adapter: None,
            device: None,
            queue: None,
            timer: None,
            frame_acquired: false,
            input_state: InputState::new(),
            #[cfg(feature = "gamepad_input")]
            gil: Gilrs::new().expect("Gamepad input initialization failed!"),
        };

        let app_handle = async_std::task::spawn(async move {
            debug!(
                "App thread: {:?} [{}]",
                std::thread::current().id(),
                std::thread::current().name().unwrap_or("UNNAMED")
            );

            loop {
                if let Ok(event) = event_rx.recv().await {
                    match event {
                        AppEvent::Resumed(surface_configuration, device, queue) => {
                            app.on_resume(&surface_configuration, &device, &queue).await;
                        }
                        AppEvent::Suspended => app.on_suspend().await,
                        AppEvent::Resize(size, device, queue) => {
                            app.on_resize(size, &device, &queue).await
                        }
                        AppEvent::Render(frame, view, device, queue) => {
                            app.on_render(&view, &device, &queue).await;

                            if let Err(e) =
                                app_change_tx.send(AppChange::FinishedRedraw(frame)).await
                            {
                                error!("Failed to send app change to app change channel: {}", e);
                            }
                        }
                        AppEvent::Update {
                            input_state,
                            delta_time,
                            cycle,
                        } => {
                            if let Some(changes) =
                                app.on_update(&input_state, delta_time, cycle).await
                            {
                                for app_change in changes {
                                    if let Err(e) = app_change_tx.send(app_change).await {
                                        error!(
                                            "Failed to send app change to app change channel: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        let result = event_loop
            .run_app(&mut app_runtime)
            .map_err(Error::EventLoopError);

        // Terminate app handle
        let _ = app_handle.cancel().now_or_never();

        result
    }

    fn make_instance() -> Instance {
        let instance = Instance::new(InstanceDescriptor {
            backends: backend_bits_from_env().unwrap_or_default(),
            flags: InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler: dx12_shader_compiler_from_env().unwrap_or_default(),
            gles_minor_version: gles_minor_version_from_env().unwrap_or_default(),
        });

        debug!("Instance: {:#?}", instance);

        instance
    }

    fn retrieve_and_rank_adapters(
        instance: &Instance,
        compatible_surface: Option<&Surface>,
    ) -> Vec<(Adapter, u128)> {
        let mut valid_adapters_ranked: Vec<_> = instance
            .enumerate_adapters(Backends::all())
            .into_iter()
            // Remove any adapters that don't support the surface
            .filter(|adapter| {
                compatible_surface.is_none()
                    || adapter.is_surface_supported(compatible_surface.unwrap())
            })
            // Initialize scoring
            .map(|adapter| (adapter, 0u128))
            // Map and match device types based on preference
            .map(|(adapter, score)| {
                let local_score = match adapter.get_info().device_type {
                    DeviceType::DiscreteGpu => 1000,
                    DeviceType::IntegratedGpu => 100,
                    DeviceType::VirtualGpu => 0,
                    DeviceType::Cpu => 0,
                    DeviceType::Other => 0,
                };

                (adapter, score + local_score)
            })
            // Map and match device backends based on preference
            .map(|(adapter, score)| {
                let local_score = match adapter.get_info().backend {
                    // DX12 and Metal should be preferred where available (i.e. on Windows and macOS) over Vulkan
                    Backend::Dx12 => 1000,
                    Backend::Metal => 1000,
                    // Vulkan is the universal default
                    Backend::Vulkan => 100,
                    // In Webbrowsers, only WebGPU should be available (or WebGL which should fall below into Backend::Gl). To prevent this from being chosen, somehow, on Desktop platforms over something more performant we set a lower score than above, but higher than OpenGL.
                    Backend::BrowserWebGpu => 50,
                    // OpenGL and Empty are not recommended at all and may not even work at all
                    Backend::Gl => 0,
                    Backend::Empty => 0,
                };

                (adapter, score + local_score)
            })
            // For each limit, increase the score.
            // Thus, higher limits == higher score.
            .map(|(adapter, score)| {
                let mut local_score = score;
                local_score += adapter.limits().max_texture_dimension_1d as u128;
                local_score += adapter.limits().max_texture_dimension_2d as u128;
                local_score += adapter.limits().max_texture_dimension_3d as u128;
                local_score += adapter.limits().max_texture_array_layers as u128;
                local_score += adapter.limits().max_bind_groups as u128;
                local_score += adapter.limits().max_bindings_per_bind_group as u128;
                local_score += adapter
                    .limits()
                    .max_dynamic_uniform_buffers_per_pipeline_layout
                    as u128;
                local_score += adapter
                    .limits()
                    .max_dynamic_storage_buffers_per_pipeline_layout
                    as u128;
                local_score += adapter.limits().max_sampled_textures_per_shader_stage as u128;
                local_score += adapter.limits().max_samplers_per_shader_stage as u128;
                local_score += adapter.limits().max_storage_buffers_per_shader_stage as u128;
                local_score += adapter.limits().max_storage_textures_per_shader_stage as u128;
                local_score += adapter.limits().max_uniform_buffers_per_shader_stage as u128;
                local_score += adapter.limits().max_uniform_buffer_binding_size as u128;
                local_score += adapter.limits().max_storage_buffer_binding_size as u128;
                local_score += adapter.limits().max_vertex_buffers as u128;
                local_score += adapter.limits().max_buffer_size as u128;
                local_score += adapter.limits().max_vertex_attributes as u128;
                local_score += adapter.limits().max_vertex_buffer_array_stride as u128;
                local_score += adapter.limits().min_uniform_buffer_offset_alignment as u128;
                local_score += adapter.limits().min_storage_buffer_offset_alignment as u128;
                local_score += adapter.limits().max_inter_stage_shader_components as u128;
                local_score += adapter.limits().max_color_attachments as u128;
                local_score += adapter.limits().max_color_attachment_bytes_per_sample as u128;
                local_score += adapter.limits().max_compute_workgroup_storage_size as u128;
                local_score += adapter.limits().max_compute_invocations_per_workgroup as u128;
                local_score += adapter.limits().max_compute_workgroup_size_x as u128;
                local_score += adapter.limits().max_compute_workgroup_size_y as u128;
                local_score += adapter.limits().max_compute_workgroup_size_z as u128;
                local_score += adapter.limits().max_compute_workgroups_per_dimension as u128;
                local_score += adapter.limits().min_subgroup_size as u128;
                local_score += adapter.limits().max_subgroup_size as u128;
                local_score += adapter.limits().max_push_constant_size as u128;
                local_score += adapter.limits().max_non_sampler_bindings as u128;

                (adapter, score + local_score)
            })
            // For each feature, increase the score.
            // Thus, more features == higher score.
            .map(|(adapter, score)| {
                let feature_count = adapter.features().iter().count();

                (adapter, score + feature_count as u128)
            })
            .collect::<Vec<_>>();

        // Sort adapters
        valid_adapters_ranked.sort_by_key(|(_adapter, score)| *score);

        if valid_adapters_ranked.is_empty() {
            panic!("No suitable GPU adapters found!");
        }

        info!("Following GPU adapters found:");
        valid_adapters_ranked
            .iter()
            .enumerate()
            .for_each(|(i, (adapter, score))| {
                info!("#{}: {} [{}]", i, adapter.get_info().name, score)
            });

        valid_adapters_ranked
    }

    fn make_device_and_queue(adapter: &Adapter) -> (Device, Queue) {
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Orbital GPU"),
                required_features: Features::default() | Features::MULTIVIEW,
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
            },
            None,
        ))
        .expect("Failed creating device from chosen adapter!");
        debug!("Device: {:?}", device);
        debug!("Queue: {:?}", queue);

        (device, queue)
    }

    fn make_surface_configuration(
        surface: &Surface,
        adapter: &Adapter,
        window_size: PhysicalSize<u32>,
        vsync_enabled: bool,
    ) -> SurfaceConfiguration {
        let caps = surface.get_capabilities(adapter);
        let mut surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: *caps
                .formats
                .first()
                .expect("Surface is required to have at least one format set!"),
            width: window_size.width,
            height: window_size.height,
            desired_maximum_frame_latency: 2,
            present_mode: *caps
                .present_modes
                .first()
                .expect("Surface is required to have at least one present mode set!"),
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface_configuration.present_mode = match vsync_enabled {
            true => PresentMode::AutoVsync,
            false => PresentMode::AutoNoVsync,
        };

        // Add SRGB view format
        surface_configuration
            .view_formats
            .push(surface_configuration.format.add_srgb_suffix());

        surface_configuration
    }

    pub fn reconfigure_surface(&mut self)
    where
        Self: Sized + Send,
    {
        self.surface.as_ref().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.surface_configuration.as_ref().unwrap(),
        );

        let config_ref = self.surface_configuration.as_ref().unwrap();

        if let Err(e) = self.event_tx.try_send(AppEvent::Resize(
            Vector2 {
                x: config_ref.width,
                y: config_ref.height,
            },
            self.device.as_ref().unwrap().clone(),
            self.queue.as_ref().unwrap().clone(),
        )) {
            error!("Failed to send resize event: {}", e);
        }
    }

    pub fn acquire_next_frame(&self) -> Option<SurfaceTexture> {
        let surface = self.surface.as_ref().unwrap();

        match surface.get_current_texture() {
            Ok(frame) => Some(frame),
            Err(e) => {
                warn!("Surface next frame acquire failed: {}", e);
                None
            }
        }
    }

    pub fn calculate_center_point(&self) {
        if let Some(window) = &self.window {
            let size = window.inner_size();

            let half_width = size.width as i32 / 2;
            let half_height = size.height as i32 / 2;

            unsafe {
                WINDOW_HALF_SIZE = (half_width, half_height);
            }
        }
    }

    pub fn redraw(&mut self) {
        // Skip if a frame is already acquired.
        // Prevents WGPU from throwing validation errors when we are
        // re-requesting the next frame, when the current frame isn't
        // presented yet.
        if self.frame_acquired {
            return;
        }

        // Check if surface and device are present
        if self.surface.is_none() || self.device.is_none() {
            warn!("Redraw requested, but runtime is in an incomplete state!");
            return;
        }

        // Get next frame to render on
        if let Some(frame) = self.acquire_next_frame() {
            if let Some(format) = self
                .surface_configuration
                .as_ref()
                .unwrap()
                .view_formats
                .first()
            {
                self.frame_acquired = true;

                let view = frame.texture.create_view(&TextureViewDescriptor {
                    format: Some(*format),
                    ..TextureViewDescriptor::default()
                });

                // Trigger Render: This is NOT directly rendering, but sending
                // an event on the message challenge to inform the App it should
                // render "now". This is NOT blocking, meaning we can't expect
                // the frame to be ready rendered immediately after.
                // Instead, we move the `frame.present()` over after App
                // actually received the event and DID the rendering!
                if let Err(e) = self.event_tx.try_send(AppEvent::Render(
                    frame,
                    view,
                    self.device.as_ref().unwrap().clone(),
                    self.queue.as_ref().unwrap().clone(),
                )) {
                    error!("Failed to send render event: {}", e);
                }
            } else {
                warn!("Surface configuration doesn't have any view formats!");
            }
        } else {
            warn!("No surface yet, but redraw was requested!");
        }
    }

    #[cfg(feature = "gamepad_input")]
    fn receive_controller_inputs(&mut self) {
        use crate::input::InputEvent;

        while let Some(gil_event) = self.gil.next_event() {
            if let Some(input_event) = InputEvent::convert_gil_event(gil_event) {
                self.input_state.handle_event(input_event);
            }
        }
    }

    fn update(&mut self) {
        let (delta_time, cycle) = self.timer.as_mut().expect("Timer went missing").tick();

        if let Some((total_delta, fps)) = cycle {
            debug!(
                "FPS: {} | TDT: {}s | CDT: {}s",
                fps, total_delta, delta_time
            );
        }

        // TODO: Needed?
        self.calculate_center_point();

        // Check for gamepad input events if the feature is enabled
        #[cfg(feature = "gamepad_input_poll")]
        self.receive_controller_inputs();

        // Trigger an update with the input state!
        if let Err(e) = self.event_tx.try_send(AppEvent::Update {
            input_state: self.input_state.clone(),
            delta_time,
            cycle,
        }) {
            error!("Failed to send update event: {}", e);
        }
    }

    fn process_app_changes(&mut self) -> bool {
        let mut exit_requested = false;

        while let Ok(app_change) = self.app_change_rx.try_recv() {
            match app_change {
                AppChange::ChangeCursorAppearance(cursor) => {
                    if let Some(window) = &self.window {
                        window.set_cursor(cursor);
                    } else {
                        warn!("Change cursor appearance requested, but window does not exist!");
                    }
                }
                AppChange::ChangeCursorPosition(position) => {
                    if let Some(window) = &self.window {
                        if let Err(e) = window.set_cursor_position(position) {
                            error!("Failed to set cursor position: {}", e);
                        }
                    } else {
                        warn!("Change cursor position requested, but window does not exist!");
                    }
                }
                AppChange::ChangeCursorVisible(visible) => {
                    if let Some(window) = &self.window {
                        window.set_cursor_visible(visible);
                    } else {
                        warn!("Change cursor visibility requested, but window does not exist!");
                    }
                }
                AppChange::ChangeCursorGrabbed(grab) => {
                    if let Some(window) = &self.window {
                        if grab {
                            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                                error!("Failed to set cursor grab! This might not be supported on your platform. Error: {}", e);
                            }
                        } else if let Err(e) = window.set_cursor_grab(CursorGrabMode::None) {
                            error!("Failed to unset cursor grab! Error: {}", e);
                        }
                    } else {
                        warn!("Change cursor grabbing requested, but window does not exist!");
                    }
                }
                AppChange::RequestAppClosure => {
                    warn!("App closure was requested!");
                    exit_requested = true;
                }
                AppChange::ForceAppClosure { exit_code } => {
                    warn!(
                        "Force app closure was requested with exit code {}!",
                        exit_code
                    );
                    std::process::exit(exit_code);
                }
                AppChange::RequestRedraw => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    } else {
                        warn!("Redraw requested, but window does not exist!");
                    }
                }
                AppChange::FinishedRedraw(frame) => {
                    frame.present();
                    self.frame_acquired = false;

                    self.input_state.reset_deltas(); // TODO: Move to update?

                    // Trigger next update cycle after the frame was fully rendered!
                    self.update();
                }
            }
        }

        exit_requested
    }
}

impl ApplicationHandler for AppRuntime {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Fill window handle and remake the device & queue chain
        self.window = Some(Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_active(true)
                        .with_inner_size(self.runtime_settings.size)
                        .with_title(self.runtime_settings.name.clone()),
                )
                .unwrap(),
        ));

        self.instance = Some(AppRuntime::make_instance());

        self.surface = Some(
            self.instance
                .as_ref()
                .unwrap()
                .create_surface(
                    self.window
                        .as_ref()
                        .expect("Expected a Window to exist by now!")
                        .clone(),
                )
                .expect("Surface creation failed"),
        );

        let mut adapters_ranked = AppRuntime::retrieve_and_rank_adapters(
            self.instance
                .as_ref()
                .expect("Expected an Instance to exist by now!"),
            self.surface.as_ref(),
        );

        let (chosen_adapter, chosen_score) = adapters_ranked.swap_remove(adapters_ranked.len() - 1);
        info!(
            "Chosen adapter: {} [{} points]\n{:?}",
            chosen_adapter.get_info().name,
            chosen_score,
            chosen_adapter.get_info()
        );
        self.adapter = Some(chosen_adapter);

        let window_size = self.window.as_ref().unwrap().inner_size();
        self.surface_configuration = Some(AppRuntime::make_surface_configuration(
            self.surface
                .as_ref()
                .expect("Expected a Surface to exist by now!"),
            self.adapter
                .as_ref()
                .expect("Expected an Adapter to exist by now!"),
            window_size,
            self.runtime_settings.vsync_enabled,
        ));

        let (device, queue) = AppRuntime::make_device_and_queue(
            self.adapter
                .as_ref()
                .expect("Expected an Adapter to be set by now!"),
        );
        self.device = Some(Arc::new(device));
        self.queue = Some(Arc::new(queue));

        self.timer = Some(Timer::new());

        self.reconfigure_surface();

        if let Err(e) = self.event_tx.try_send(AppEvent::Resumed(
            self.surface_configuration
                .as_ref()
                .expect("Expected a SurfaceConfiguration to exist by now!")
                .clone(),
            self.device
                .as_ref()
                .expect("Expected a Device to exist by now!")
                .clone(),
            self.queue
                .as_ref()
                .expect("Expected a Queue to exist by now!")
                .clone(),
        )) {
            error!("Failed to send WindowEvent to App: {}", e);
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Invalidate everything related to the window, surface and device.
        self.window = None;
        self.surface = None;
        self.surface_configuration = None;
        self.instance = None;
        self.adapter = None;
        self.device = None;
        self.queue = None;
        self.timer = None;

        if let Err(e) = self.event_tx.try_send(AppEvent::Suspended) {
            error!("Failed to send Suspend Event to App: {}", e);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Skip if exiting
        if event_loop.exiting() {
            return;
        }

        let input_event = match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                None
            }
            WindowEvent::RedrawRequested => {
                self.redraw();

                #[cfg(feature = "auto_request_redraw")]
                self.window.as_ref().unwrap().request_redraw();

                if self.process_app_changes() {
                    event_loop.exit();
                }

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
            } => Some(InputEvent::MouseMovedPosition {
                device_id,
                position,
            }),
            WindowEvent::Resized(new_size) => {
                self.surface_configuration = Some(AppRuntime::make_surface_configuration(
                    self.surface.as_ref().unwrap(),
                    self.adapter.as_ref().unwrap(),
                    new_size,
                    self.runtime_settings.vsync_enabled,
                ));

                self.reconfigure_surface();

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
        if let Some(input_event) = InputEvent::convert_device_event(device_id, device_event) {
            self.input_state.handle_event(input_event);
        }
    }
}
