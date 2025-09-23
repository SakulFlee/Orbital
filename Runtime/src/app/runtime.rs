use std::mem::transmute;
use std::thread;

use async_std::task::block_on;
use cgmath::Vector2;
use gilrs::Gilrs;
use wgpu::{
    Adapter, Backend, BackendOptions, Backends, CompositeAlphaMode, Device, DeviceDescriptor,
    DeviceType, Features, Instance, InstanceDescriptor, InstanceFlags, Limits,
    MemoryBudgetThresholds, MemoryHints, PresentMode, Queue, Surface, SurfaceConfiguration,
    SurfaceError, SurfaceTexture, TextureUsages, TextureViewDescriptor, Trace,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window, WindowId},
};

use super::{
    input::{InputEvent, InputState},
    Timer,
};
use super::{App, AppSettings};
use crate::{
    app::AppEvent,
    element::Element,
    element::Message,
    logging::{self, debug, error, info, warn},
};

pub struct AppRuntime<AppImpl: App> {
    app: AppImpl,
    app_messages: Vec<Message>,
    // App related
    runtime_settings: AppSettings,
    // Window related
    window: Option<Window>,
    surface: Option<Surface<'static>>,
    surface_configuration: Option<SurfaceConfiguration>,
    // Device related
    instance: Option<Instance>,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
    timer: Option<Timer>,
    input_state: InputState,
    #[cfg(feature = "gamepad_input")]
    gil: Gilrs,
}

impl<AppImpl: App> AppRuntime<AppImpl> {
    pub fn liftoff(
        event_loop: EventLoop<()>,
        settings: AppSettings,
        mut app: AppImpl,
    ) -> Result<(), EventLoopError> {
        logging::init();

        info!("Orbital Runtime");
        info!(" --- @SakulFlee --- ");

        block_on(app.on_startup());

        let mut app_runtime = Self {
            app,
            app_messages: Vec::new(),
            runtime_settings: settings,
            window: None,
            surface: None,
            surface_configuration: None,
            instance: None,
            adapter: None,
            device: None,
            queue: None,
            timer: None,
            input_state: InputState::new(),
            #[cfg(feature = "gamepad_input")]
            gil: Gilrs::new().expect("Gamepad input initialization failed!"),
        };

        event_loop.run_app(&mut app_runtime)
    }

    fn make_instance() -> Instance {
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::from_env().unwrap_or(Backends::all()),
            flags: InstanceFlags::from_build_config().with_env(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::from_env_or_default(),
        });

        debug!("Instance: {instance:#?}");

        instance
    }

    fn retrieve_and_rank_adapters(
        instance: &Instance,
        compatible_surface: Option<&Surface>,
    ) -> Vec<(Adapter, (u8, u8, u128, usize))> {
        let mut valid_adapters_ranked: Vec<_> = instance
            .enumerate_adapters(Backends::all())
            .into_iter()
            // Remove any adapters that don't support the surface
            .filter(|adapter| {
                compatible_surface.is_none()
                    || adapter.is_surface_supported(compatible_surface.unwrap())
            })
            // Initialize scoring
            .map(|adapter| (adapter, (0u8, 0u8, 0u128, 0usize)))
            // Map and match device types based on preference
            .map(|(adapter, pri)| {
                let device_pri = match adapter.get_info().device_type {
                    DeviceType::DiscreteGpu => 3,
                    DeviceType::IntegratedGpu => 2,
                    DeviceType::VirtualGpu => 1,
                    DeviceType::Cpu => 0,
                    DeviceType::Other => 0,
                };

                (adapter, (device_pri, pri.1, pri.2, pri.3))
            })
            // Map and match device backends based on preference
            .map(|(adapter, pri)| {
                let backend_pri = match adapter.get_info().backend {
                    // DX12 and Metal should be preferred where available (i.e. on Windows and macOS) over Vulkan
                    Backend::Dx12 => 4,
                    Backend::Metal => 4,
                    // Vulkan is the universal default
                    Backend::Vulkan => 3,
                    // In Webbrowsers, only WebGPU should be available (or WebGL which should fall below into Backend::Gl). To prevent this from being chosen, somehow, on Desktop platforms over something more performant we set a lower score than above, but higher than OpenGL.
                    Backend::BrowserWebGpu => 2,
                    // OpenGL and Empty are not recommended at all and may not even work at all
                    Backend::Gl => 1,
                    Backend::Noop => 0,
                };

                (adapter, (pri.0, backend_pri, pri.2, pri.3))
            })
            // For each limit, increase the score.
            // Thus, higher limits == higher score.
            .map(|(adapter, pri)| {
                let mut limits_sum = 0u128;
                limits_sum += adapter.limits().max_texture_dimension_1d as u128;
                limits_sum += adapter.limits().max_texture_dimension_2d as u128;
                limits_sum += adapter.limits().max_texture_dimension_3d as u128;
                limits_sum += adapter.limits().max_texture_array_layers as u128;
                limits_sum += adapter.limits().max_bind_groups as u128;
                limits_sum += adapter.limits().max_bindings_per_bind_group as u128;
                limits_sum += adapter
                    .limits()
                    .max_dynamic_uniform_buffers_per_pipeline_layout
                    as u128;
                limits_sum += adapter
                    .limits()
                    .max_dynamic_storage_buffers_per_pipeline_layout
                    as u128;
                limits_sum += adapter.limits().max_sampled_textures_per_shader_stage as u128;
                limits_sum += adapter.limits().max_samplers_per_shader_stage as u128;
                limits_sum += adapter.limits().max_storage_buffers_per_shader_stage as u128;
                limits_sum += adapter.limits().max_storage_textures_per_shader_stage as u128;
                limits_sum += adapter.limits().max_uniform_buffers_per_shader_stage as u128;
                limits_sum += adapter.limits().max_uniform_buffer_binding_size as u128;
                limits_sum += adapter.limits().max_storage_buffer_binding_size as u128;
                limits_sum += adapter.limits().max_vertex_buffers as u128;
                limits_sum += adapter.limits().max_buffer_size as u128;
                limits_sum += adapter.limits().max_vertex_attributes as u128;
                limits_sum += adapter.limits().max_vertex_buffer_array_stride as u128;
                limits_sum += adapter.limits().min_uniform_buffer_offset_alignment as u128;
                limits_sum += adapter.limits().min_storage_buffer_offset_alignment as u128;
                limits_sum += adapter.limits().max_inter_stage_shader_components as u128;
                limits_sum += adapter.limits().max_color_attachments as u128;
                limits_sum += adapter.limits().max_color_attachment_bytes_per_sample as u128;
                limits_sum += adapter.limits().max_compute_workgroup_storage_size as u128;
                limits_sum += adapter.limits().max_compute_invocations_per_workgroup as u128;
                limits_sum += adapter.limits().max_compute_workgroup_size_x as u128;
                limits_sum += adapter.limits().max_compute_workgroup_size_y as u128;
                limits_sum += adapter.limits().max_compute_workgroup_size_z as u128;
                limits_sum += adapter.limits().max_compute_workgroups_per_dimension as u128;
                limits_sum += adapter.limits().min_subgroup_size as u128;
                limits_sum += adapter.limits().max_subgroup_size as u128;
                limits_sum += adapter.limits().max_push_constant_size as u128;
                limits_sum += adapter.limits().max_non_sampler_bindings as u128;

                (adapter, (pri.0, pri.1, limits_sum, pri.3))
            })
            // For each feature, increase the score.
            // Thus, more features == higher score.
            .map(|(adapter, pri)| {
                let features_count = adapter.features().iter().count();

                (adapter, (pri.0, pri.1, pri.2, features_count))
            })
            .collect::<Vec<_>>();

        // Sort adapters
        valid_adapters_ranked.sort_by(|a, b| b.1.cmp(&a.1));

        if valid_adapters_ranked.is_empty() {
            panic!("No suitable GPU adapters found!");
        }

        info!("Following GPU adapters found:");
        valid_adapters_ranked
            .iter()
            .enumerate()
            .for_each(|(i, (adapter, score))| {
                info!("#{}: {} {:?}", i, adapter.get_info().name, score)
            });

        valid_adapters_ranked
    }

    fn make_device_and_queue(adapter: &Adapter) -> (Device, Queue) {
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("Orbital GPU"),
            required_features: Features::default() | Features::POLYGON_MODE_LINE,
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        }))
        .expect("Failed creating device from chosen adapter!");
        debug!("Device: {device:?}");
        debug!("Queue: {queue:?}");

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

        block_on(self.app.on_resize(
            Vector2 {
                x: config_ref.width,
                y: config_ref.height,
            },
            self.device.as_ref().unwrap(),
            self.queue.as_ref().unwrap(),
        ));
    }

    pub fn acquire_next_frame(&mut self) -> Result<SurfaceTexture, SurfaceError> {
        let surface = self.surface.as_ref().unwrap();

        surface.get_current_texture()
    }

    pub fn redraw(&mut self) {
        // TODO: Not sure if still needed after sync-change?
        // // Skip if a frame is already acquired.
        // // Prevents WGPU from throwing validation errors when we are
        // // re-requesting the next frame, when the current frame isn't
        // // presented yet.
        // if self.frame_acquired {
        //     return;
        // }

        // Check if surface and device are present
        if self.surface.is_none() || self.device.is_none() {
            warn!("Redraw requested, but runtime is in an incomplete state!");
            return;
        }

        // Get next frame to render on
        let frame = match self.acquire_next_frame() {
            Ok(surface_texture) => surface_texture,
            Err(e) => {
                warn!("Failed to acquire next frame from surface: {e}");

                warn!("Attempting reconfiguration ...");
                self.reconfigure_surface();

                match self.acquire_next_frame() {
                    Ok(surface_texture) => surface_texture,
                    Err(e) => panic!(
                        "Failed to acquire next frame from surface after reconfiguration! ({e:?})"
                    ),
                }
            }
        };

        let format = match self
            .surface_configuration
            .as_ref()
            .unwrap()
            .view_formats
            .first()
        {
            Some(format) => format,
            None => {
                warn!("No view formats available for surface!");

                warn!("Attempting reconfiguration ...");
                self.reconfigure_surface();

                match self
                    .surface_configuration
                    .as_ref()
                    .unwrap()
                    .view_formats
                    .first()
                {
                    Some(format) => format,
                    None => panic!("No view formats available for surface after reconfiguration!"),
                }
            }
        };

        // TODO: Not sure if still needed!
        // self.frame_acquired = true;

        let view: wgpu::TextureView = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(*format),
            ..TextureViewDescriptor::default()
        });

        block_on(self.app.on_render(
            &view,
            self.device.as_ref().unwrap(),
            self.queue.as_ref().unwrap(),
        ));

        frame.present();
    }

    #[cfg(feature = "gamepad_input")]
    fn receive_controller_inputs(&mut self) {
        use super::input::InputEvent;

        while let Some(gil_event) = self.gil.next_event() {
            if let Some(input_event) = InputEvent::convert_gil_event(gil_event) {
                self.input_state.handle_event(input_event);
            }
        }
    }

    fn update(&mut self) -> bool {
        let (delta_time, cycle) = self.timer.as_mut().expect("Timer went missing").tick();

        if let Some((total_delta, fps)) = cycle {
            debug!("FPS: {fps} | TDT: {total_delta}s | CDT: {delta_time}s");
        }

        // Check for gamepad input events if the feature is enabled
        #[cfg(feature = "gamepad_input_poll")]
        self.receive_controller_inputs();

        let result = if let Some(app_events) =
            block_on(self.app.on_update(&self.input_state, delta_time, cycle))
        {
            self.process_app_events(app_events)
        } else {
            false
        };

        self.input_state.reset_deltas();

        result
    }

    fn process_app_events(&mut self, app_events: Vec<AppEvent>) -> bool {
        let mut exit_requested = false;

        for event in app_events {
            match event {
                AppEvent::ChangeCursorAppearance(cursor) => {
                    if let Some(window) = &self.window {
                        window.set_cursor(cursor);
                    } else {
                        warn!("Change cursor appearance requested, but window does not exist!");
                    }
                }
                AppEvent::ChangeCursorPosition(position) => {
                    if let Some(window) = &self.window {
                        if let Err(e) = window.set_cursor_position(position) {
                            error!("Failed to set cursor position: {e}");
                        }
                    } else {
                        warn!("Change cursor position requested, but window does not exist!");
                    }
                }
                AppEvent::ChangeCursorVisible(visible) => {
                    if let Some(window) = &self.window {
                        window.set_cursor_visible(visible);
                    } else {
                        warn!("Change cursor visibility requested, but window does not exist!");
                    }
                }
                AppEvent::ChangeCursorGrabbed(grab) => {
                    if let Some(window) = &self.window {
                        if grab {
                            if let Err(e) = window.set_cursor_grab(CursorGrabMode::Confined) {
                                error!(
                                    "Failed to set cursor grab! This might not be supported on your platform. Error: {e}"
                                );
                            }
                        } else if let Err(e) = window.set_cursor_grab(CursorGrabMode::None) {
                            error!("Failed to unset cursor grab! Error: {e}");
                        }
                    } else {
                        warn!("Change cursor grabbing requested, but window does not exist!");
                    }
                }
                AppEvent::RequestAppClosure => {
                    warn!("App closure was requested!");
                    exit_requested = true;
                }
                AppEvent::ForceAppClosure { exit_code } => {
                    warn!("Force app closure was requested with exit code {exit_code}!");
                    std::process::exit(exit_code);
                }
                AppEvent::RequestRedraw => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    } else {
                        warn!("Redraw requested, but window does not exist!");
                    }
                }
                AppEvent::SendMessage(message) => {
                    self.app_messages.push(message);
                }
            }
        }

        exit_requested
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        // Signal the application to close without forcing immediate cleanup
        // This allows the event loop to shut down gracefully
        event_loop.exit();
    }
}

impl<AppImpl: App> ApplicationHandler for AppRuntime<AppImpl> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Remake all window, device, queue, etc. related structures

        self.window = Some(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_active(true)
                        .with_inner_size(self.runtime_settings.size)
                        .with_title(self.runtime_settings.name.clone()),
                )
                .unwrap(),
        );

        self.instance = Some(AppRuntime::<AppImpl>::make_instance());

        self.surface = Some(unsafe {
            transmute(
                self.instance
                    .as_ref()
                    .unwrap()
                    .create_surface(self.window.as_ref().unwrap())
                    .expect("Failed creating Surface!"),
            )
        });

        let mut adapters_ranked = AppRuntime::<AppImpl>::retrieve_and_rank_adapters(
            self.instance
                .as_ref()
                .expect("Expected an Instance to exist by now!"),
            self.surface.as_ref(),
        );

        let (chosen_adapter, chosen_score) = adapters_ranked.swap_remove(adapters_ranked.len() - 1);
        info!(
            "Chosen adapter: {} {:?}\n{:?}",
            chosen_adapter.get_info().name,
            chosen_score,
            chosen_adapter.get_info()
        );
        self.adapter = Some(chosen_adapter);

        let window_size = self.window.as_ref().unwrap().inner_size();
        self.surface_configuration = Some(AppRuntime::<AppImpl>::make_surface_configuration(
            self.surface
                .as_ref()
                .expect("Expected a Surface to exist by now!"),
            self.adapter
                .as_ref()
                .expect("Expected an Adapter to exist by now!"),
            window_size,
            self.runtime_settings.vsync_enabled,
        ));

        let (device, queue) = AppRuntime::<AppImpl>::make_device_and_queue(
            self.adapter
                .as_ref()
                .expect("Expected an Adapter to be set by now!"),
        );
        self.device = Some(device);
        self.queue = Some(queue);

        self.timer = Some(Timer::new());

        self.reconfigure_surface();

        block_on(
            self.app.on_resume(
                self.surface_configuration
                    .as_ref()
                    .expect("SurfaceConfiguration must exist at this point!"),
                self.device
                    .as_ref()
                    .expect("Device must exist at this point!"),
                self.queue
                    .as_ref()
                    .expect("Queue must exist at this point!"),
            ),
        );
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        debug!("Suspending application...");

        // Call app.on_suspend() first
        debug!("Calling app.on_suspend()...");
        block_on(self.app.on_suspend());
        debug!("App.on_suspend() completed.");

        // Add a small delay to ensure app.on_suspend() has completed
        debug!("Waiting for app.on_suspend() to complete...");
        thread::sleep(std::time::Duration::from_millis(100));

        // Invalidate everything related to the window, surface and device.
        // Important: Drop resources in the correct order with delays to prevent segfaults
        debug!("Dropping all resources...");
        self.surface_configuration = None;

        // Drop the surface before the device to prevent Vulkan validation errors
        self.surface = None;

        // Add a small delay before dropping device to ensure all GPU operations complete
        thread::sleep(std::time::Duration::from_millis(50));

        self.queue = None;
        self.device = None;
        self.adapter = None;
        self.instance = None;
        self.window = None;
        self.timer = None;

        debug!("Suspension complete.");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Skip if exiting
        if event_loop.exiting() {
            debug!("EventLoop marked exiting! Further events will be skipped ...");
            return;
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
                self.window.as_ref().unwrap().request_redraw();

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
                self.surface_configuration =
                    Some(AppRuntime::<AppImpl>::make_surface_configuration(
                        self.surface.as_ref().unwrap(),
                        self.adapter.as_ref().unwrap(),
                        new_size,
                        self.runtime_settings.vsync_enabled,
                    ));

                self.reconfigure_surface();

                self.input_state.surface_resize(new_size);

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
