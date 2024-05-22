use std::sync::Arc;

use log::{debug, info, warn};
use wgpu::{
    util::{
        backend_bits_from_env, dx12_shader_compiler_from_env, gles_minor_version_from_env,
        initialize_adapter_from_env_or_default,
    },
    Adapter, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, InstanceFlags,
    Limits, Queue, Surface, SurfaceConfiguration, SurfaceTexture, TextureViewDescriptor,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub mod settings;
pub use settings::*;

pub mod app;
pub use app::*;

use crate::error::Error;

#[derive(Default)]
pub struct Runtime<AppImpl: App> {
    // App related
    app: Option<AppImpl>,
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

impl<AppImpl: App> Runtime<AppImpl> {
    pub fn liftoff(event_loop: EventLoop<()>, _settings: RuntimeSettings) -> Result<(), Error> {
        info!("Akimo-Project: Runtime");
        info!("(C) SakulFlee 2024");

        let mut runtime = Self {
            app: None,
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
        let adapter = pollster::block_on(initialize_adapter_from_env_or_default(
            instance,
            compatible_surface,
        ))
        .expect("No suitable GPU adapters found!");

        let adapter_info = adapter.get_info();
        debug!("Adapter: {} ({:#?})", adapter_info.name, adapter_info);

        adapter
    }

    fn make_surface_configuration(
        surface: &Surface,
        adapter: &Adapter,
        window_size: PhysicalSize<u32>,
    ) -> SurfaceConfiguration {
        let mut surface_configuration = surface
            .get_default_config(adapter, window_size.width, window_size.height)
            .expect("Surface isn't compatible with chosen adapter!");

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

        debug!(
            "View formats: {:#?}",
            self.surface_configuration.as_ref().unwrap().view_formats
        ); // FIXME
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
                    .render(
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
}

impl<AppImpl: App> ApplicationHandler for Runtime<AppImpl> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Fill window handle and remake the device & queue chain
        self.window = Some(Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_active(true)
                        .with_inner_size(PhysicalSize::new(1280, 720)),
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
        ));

        let (device, queue) = pollster::block_on(self.adapter.as_ref().unwrap().request_device(
            &DeviceDescriptor {
                label: None,
                required_features: Features::default(),
                required_limits: Limits::default(),
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
        match event {
            WindowEvent::Resized(new_size) => {
                self.surface_configuration = Some(Self::make_surface_configuration(
                    self.surface.as_ref().unwrap(),
                    self.adapter.as_ref().unwrap(),
                    new_size,
                ));

                self.reconfigure_surface();
            }
            WindowEvent::CloseRequested => {
                info!("Close requested!");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.redraw();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => debug!("Unhandled WindowEvent encountered: {:#?}", event),
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        debug!("New Events: {:#?}", cause);

        match self.app.as_mut() {
            Some(app) => app.update(),
            None => warn!("App not present in Runtime! Skipping update."),
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        warn!("Memory warning received!");
    }
}
