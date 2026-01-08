use std::{error::Error, mem::transmute};

use async_std::task::block_on;
use log::debug;
use wgpu::{
    Adapter, BackendOptions, Backends, CompositeAlphaMode, CreateSurfaceError, Device,
    DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, InstanceFlags,
    Limits, MemoryBudgetThresholds, MemoryHints, PowerPreference, PresentMode, Queue,
    RequestAdapterError, RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, Trace,
};
use winit::{dpi::Size, error::OsError, event_loop::ActiveEventLoop, window::Window};

use crate::app::AppSettings;

pub type AppCtx = AppContext;

#[derive(Debug)]
pub struct AppContext {
    window: Window,
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
}

impl AppContext {
    pub fn new(
        event_loop: &ActiveEventLoop,
        settings: AppSettings,
    ) -> Result<Self, Box<dyn Error>> {
        let window = Self::make_window(event_loop, settings.size, settings.name)?;
        debug!("Window: {:?}", window);

        let instance = Self::make_instance();
        debug!("Instance: {:?}", instance);

        let surface = Self::make_surface(&instance, &window)?;
        debug!("Surface: {:?}", surface);

        let adapter = Self::make_adapter(&instance, &surface)?;
        debug!("Adapter: {:?}", adapter);

        let (device, queue) = Self::make_device_and_queue(&adapter)?;
        debug!("Device: {:?}", device);
        debug!("Queue: {:?}", queue);

        Ok(Self {
            window,
            instance,
            adapter,
            device,
            queue,
            surface,
        })
    }

    fn make_window(
        event_loop: &ActiveEventLoop,
        inner_size: Size,
        title: String,
    ) -> Result<Window, OsError> {
        event_loop.create_window(
            Window::default_attributes()
                .with_active(true)
                .with_inner_size(inner_size)
                .with_title(title),
        )
    }

    fn make_instance() -> Instance {
        Instance::new(&InstanceDescriptor {
            // Check for environment variables, otherwise revert to using all backends.
            backends: Backends::from_env().unwrap_or(Backends::all()),
            // Enables debugging flags only for debug builds.
            flags: InstanceFlags::from_build_config(),
            // Choose backends options from environment variables, otherwise use defaults.
            backend_options: BackendOptions::from_env_or_default(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
        })
    }

    fn make_surface(
        instance: &Instance,
        window: &Window,
    ) -> Result<Surface<'static>, CreateSurfaceError> {
        unsafe { transmute(instance.create_surface(window)) }
    }

    fn make_adapter(
        instance: &Instance,
        surface: &Surface,
    ) -> Result<Adapter, RequestAdapterError> {
        block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        }))
    }

    fn make_device_and_queue(adapter: &Adapter) -> Result<(Device, Queue), RequestDeviceError> {
        block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("Orbital GPU"),
            required_features: Features::default() | Features::POLYGON_MODE_LINE,
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
            experimental_features: ExperimentalFeatures::disabled(),
        }))
    }

    pub fn make_surface_configuration(&self, vsync: bool) -> SurfaceConfiguration {
        let capabilities = self.surface.get_capabilities(&self.adapter);

        let present_mode = match vsync {
            true => PresentMode::AutoVsync,
            false => PresentMode::AutoNoVsync,
        };

        let window_size = self.window.inner_size();

        let mut view_formats = capabilities.formats;
        let first_format = view_formats
            .first()
            .expect("There must be at least one surface format!");
        let srgb_format = first_format.add_srgb_suffix();
        view_formats.insert(0, srgb_format);

        let mut default_config = self
            .surface
            .get_default_config(&self.adapter, window_size.width, window_size.height)
            .unwrap_or(SurfaceConfiguration {
                usage: TextureUsages::empty(),
                format: TextureFormat::Rgba8UnormSrgb,
                width: 100,
                height: 100,
                present_mode,
                desired_maximum_frame_latency: 2,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
            });

        default_config.desired_maximum_frame_latency = 2;
        default_config.present_mode;
        default_config.alpha_mode = CompositeAlphaMode::Auto;
        default_config.format = srgb_format;
        default_config.usage = TextureUsages::RENDER_ATTACHMENT;
        default_config.view_formats = view_formats;
        default_config.width = window_size.width;
        default_config.height = window_size.height;
        default_config.desired_maximum_frame_latency = 2;

        default_config
    }

    pub fn reconfigure_surface(&self, configuration: &SurfaceConfiguration) {
        self.surface.configure(&self.device, configuration);
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn instance_mut(&mut self) -> &mut Instance {
        &mut self.instance
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn adapter_mut(&mut self) -> &mut Adapter {
        &mut self.adapter
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn device_mut(&mut self) -> &mut Device {
        &mut self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn queue_mut(&mut self) -> &mut Queue {
        &mut self.queue
    }

    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    pub fn surface_mut(&mut self) -> &mut Surface<'static> {
        &mut self.surface
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}
