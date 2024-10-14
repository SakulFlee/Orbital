use std::sync::Arc;

use cgmath::Vector2;
use gilrs::Gilrs;
use log::{debug, error, info, warn};
use wgpu::{
    util::{backend_bits_from_env, dx12_shader_compiler_from_env, gles_minor_version_from_env},
    Adapter, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, InstanceFlags,
    Limits, MemoryHints, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::error::Error;

use super::{App, AppChange, AppSettings, InputEvent};

pub struct AppRuntime<AppImpl: App> {
    // App related
    app: Option<AppImpl>,
    runtime_settings: AppSettings,
    gil: Gilrs,
    // Window related
    window: Option<Arc<Window>>,
    surface: Option<Surface<'static>>,
    surface_configuration: Option<SurfaceConfiguration>,
    // Device related
    instance: Option<Instance>,
    adapter: Option<Adapter>,
    device: Option<Device>,
    queue: Option<Queue>,
}

pub static mut WINDOW_HALF_SIZE: (i32, i32) = (0, 0);

impl<AppImpl: App> AppRuntime<AppImpl> {
    pub fn liftoff(event_loop: EventLoop<()>, settings: AppSettings) -> Result<(), Error> {
        info!("Orbital: App Runtime");
        info!(" --- @SakulFlee --- ");

        Self::__liftoff(event_loop, settings)
    }

    pub(crate) fn __liftoff(
        event_loop: EventLoop<()>,
        runtime_settings: AppSettings,
    ) -> Result<(), Error> {
        let mut runtime = Self {
            app: None,
            runtime_settings,
            gil: Gilrs::new().unwrap(),
            window: None,
            surface: None,
            surface_configuration: None,
            instance: None,
            adapter: None,
            device: None,
            queue: None,
        };

        event_loop
            .run_app(&mut runtime)
            .map_err(Error::EventLoopError)
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

    fn make_adapter(instance: &Instance, compatible_surface: Option<&Surface>) -> Adapter {
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface,
        }))
        .expect("No suitable GPU adapters found!");

        let adapter_info = adapter.get_info();
        debug!("Adapter: {} ({:#?})", adapter_info.name, adapter_info);

        adapter
    }

    fn make_surface_configuration(
        surface: &Surface,
        adapter: &Adapter,
        window_size: PhysicalSize<u32>,
        vsync_enabled: bool,
    ) -> SurfaceConfiguration {
        let mut surface_configuration = surface
            .get_default_config(adapter, window_size.width, window_size.height)
            .expect("Surface isn't compatible with chosen adapter!");

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

    pub fn reconfigure_surface(&mut self) {
        self.surface.as_ref().unwrap().configure(
            self.device.as_ref().unwrap(),
            self.surface_configuration.as_ref().unwrap(),
        );

        if let Some(app) = &mut self.app {
            let config_ref = self.surface_configuration.as_ref().unwrap();

            app.on_resize(
                Vector2 {
                    x: config_ref.width,
                    y: config_ref.height,
                },
                self.device.as_ref().unwrap(),
                self.queue.as_ref().unwrap(),
            )
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
                let view = frame.texture.create_view(&TextureViewDescriptor {
                    format: Some(*format),
                    ..TextureViewDescriptor::default()
                });

                // Render!
                self.app
                    .as_mut()
                    .expect("Redraw on runtime without app")
                    .on_render(
                        &view,
                        self.device.as_ref().unwrap(),
                        self.queue.as_ref().unwrap(),
                    );

                // Present the frame after rendering and inform the window about a redraw being needed
                frame.present();
            } else {
                warn!("Surface configuration doesn't have any view formats!");
            }
        } else {
            warn!("No surface yet, but redraw was requested!");
        }
    }

    fn gamepad_inputs(&mut self) {
        if let Some(app) = &mut self.app {
            while let Some(gil_event) = self.gil.next_event() {
                if let Some(input_event) = InputEvent::convert(gil_event) {
                    app.on_input(&input_event);
                }
            }
        }
    }

    fn update(&mut self) -> bool {
        self.gamepad_inputs();
        self.calculate_center_point();

        let mut app_changes = Vec::new();
        if let Some(app) = self.app.as_mut() {
            let option_changes = app.on_update();

            if let Some(proposed_changes) = option_changes {
                app_changes.extend(proposed_changes);
            }
        } else {
            warn!("App not present in Runtime! Skipping update.")
        }

        for app_change in app_changes {
            match app_change {
                AppChange::RequestAppClosure => {
                    return true;
                }
                AppChange::ForceAppClosure { exit_code } => {
                    std::process::exit(exit_code);
                }
                AppChange::ChangeCursorAppearance(x) => {
                    if let Some(window) = &mut self.window {
                        window.set_cursor(x);
                    } else {
                        error!("AppChange::ChangeCursorAppearance proposed, but Window does not exist yet!");
                    }
                }
                AppChange::ChangeCursorPosition(x) => {
                    if let Some(window) = &mut self.window {
                        if let Err(e) = window.set_cursor_position(x) {
                            error!("AppChange::ChangeCursorPosition failed to change cursor position due to an external error: {}", e);
                        }
                    } else {
                        error!("AppChange::ChangeCursorPosition proposed, but Window does not exist yet!");
                    }
                }
                AppChange::ChangeCursorVisible(x) => {
                    if let Some(window) = &mut self.window {
                        window.set_cursor_visible(x);
                    } else {
                        error!("AppChange::ChangeCursorVisible proposed, but Window does not exist yet!");
                    }
                }
                AppChange::ChangeCursorGrabbed(x) => {
                    if let Some(window) = &mut self.window {
                        if x {
                            if let Err(e) = window
                                .set_cursor_grab(CursorGrabMode::Confined)
                                .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked))
                            {
                                error!("AppChange::ChangeCursorGrabbed failed to grab cursor due to an external error: {}", e);
                            }
                        } else if let Err(e) = window.set_cursor_grab(CursorGrabMode::None) {
                            error!("AppChange::ChangeCursorGrabbed failed to release cursor due to an external error: {}", e);
                        }
                    } else {
                        error!("AppChange::ChangeCursorVisible proposed, but Window does not exist yet!");
                    }
                }
            }
        }

        false
    }
}

impl<AppImpl: App> ApplicationHandler for AppRuntime<AppImpl> {
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

        self.instance = Some(Self::make_instance());

        self.surface = Some(
            self.instance
                .as_ref()
                .unwrap()
                .create_surface(self.window.as_ref().unwrap().clone())
                .expect("Surface creation failed"),
        );

        self.adapter = Some(Self::make_adapter(
            self.instance.as_ref().unwrap(),
            Some(self.surface.as_ref().unwrap()),
        ));

        let window_size = self.window.as_ref().unwrap().inner_size();
        self.surface_configuration = Some(Self::make_surface_configuration(
            self.surface.as_ref().unwrap(),
            self.adapter.as_ref().unwrap(),
            window_size,
            self.runtime_settings.vsync_enabled,
        ));

        let (device, queue) = pollster::block_on(self.adapter.as_ref().unwrap().request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
            },
            None,
        ))
        .expect("Unable to find suitable GPU device!");
        self.device = Some(device);
        self.queue = Some(queue);

        self.reconfigure_surface();

        // Check if the app exists. If not, create it.
        if self.app.is_none() {
            info!("Bootstrapping app ...");

            self.app = Some(AppImpl::init(
                self.surface_configuration.as_ref().unwrap(),
                self.device.as_ref().unwrap(),
                self.queue.as_ref().unwrap(),
            ));
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Invalidate everything related to the window, surface and device.
        // (Except for the app!)
        self.window = None;
        self.surface = None;
        self.surface_configuration = None;
        self.instance = None;
        self.adapter = None;
        self.device = None;
        self.queue = None;
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

        match event {
            WindowEvent::Resized(new_size) => {
                self.surface_configuration = Some(Self::make_surface_configuration(
                    self.surface.as_ref().unwrap(),
                    self.adapter.as_ref().unwrap(),
                    new_size,
                    self.runtime_settings.vsync_enabled,
                ));

                self.reconfigure_surface();
            }
            WindowEvent::CloseRequested => {
                info!("Close requested!");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let close_request = self.update();
                if close_request {
                    event_loop.exit();
                    return;
                }

                self.redraw();

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Focused(focused) => {
                if let Some(app) = &mut self.app {
                    app.on_focus_change(focused);
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if let Some(app) = &mut self.app {
                    app.on_input(&InputEvent::KeyboardButton {
                        device_id,
                        event,
                        is_synthetic,
                    })
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                if let Some(app) = &mut self.app {
                    app.on_input(&InputEvent::MouseButton {
                        device_id,
                        state,
                        button,
                    })
                }
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                if let Some(app) = &mut self.app {
                    app.on_input(&InputEvent::MouseWheel {
                        device_id,
                        delta,
                        phase,
                    })
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Some(app) = &mut self.app {
                    app.on_input(&InputEvent::MouseMovedPosition {
                        device_id,
                        position,
                    })
                }
            }
            WindowEvent::ModifiersChanged(_) => (),
            WindowEvent::ActivationTokenDone {
                serial: _,
                token: _,
            } => (),
            WindowEvent::Moved(_) => (),
            WindowEvent::Destroyed => (),
            WindowEvent::DroppedFile(_) => (),
            WindowEvent::HoveredFile(_) => (),
            WindowEvent::HoveredFileCancelled => (),
            WindowEvent::Ime(_) => (),
            WindowEvent::CursorEntered { device_id: _ } => (),
            WindowEvent::CursorLeft { device_id: _ } => (),
            WindowEvent::PinchGesture {
                device_id: _,
                delta: _,
                phase: _,
            } => (),
            WindowEvent::PanGesture {
                device_id: _,
                delta: _,
                phase: _,
            } => (),
            WindowEvent::DoubleTapGesture { device_id: _ } => (),
            WindowEvent::RotationGesture {
                device_id: _,
                delta: _,
                phase: _,
            } => (),
            WindowEvent::TouchpadPressure {
                device_id: _,
                pressure: _,
                stage: _,
            } => (),
            WindowEvent::AxisMotion {
                device_id: _,
                axis: _,
                value: _,
            } => (),
            WindowEvent::Touch(_) => (),
            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _,
            } => (),
            WindowEvent::ThemeChanged(_) => (),
            WindowEvent::Occluded(_) => (),
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, start_cause: StartCause) {
        if let Some(app) = self.app.as_mut() {
            app.on_next_event_cycle(start_cause);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(app) = self.app.as_mut() {
            app.on_device_event(device_id, event);
        }
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(app) = self.app.as_mut() {
            app.on_memory_warning();
        }
    }
}

impl<AppImpl: App> Default for AppRuntime<AppImpl> {
    fn default() -> Self {
        Self {
            app: Default::default(),
            runtime_settings: Default::default(),
            gil: Gilrs::new().unwrap(),
            window: Default::default(),
            surface: Default::default(),
            surface_configuration: Default::default(),
            instance: Default::default(),
            adapter: Default::default(),
            device: Default::default(),
            queue: Default::default(),
        }
    }
}
