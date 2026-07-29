use std::{error::Error, mem::transmute};

use log::debug;
use orbital_core::wgpu_util::block_on;
use wgpu::{
    Adapter, BackendOptions, Backends, CompositeAlphaMode, CreateSurfaceError,
    CurrentSurfaceTexture, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints,
    PowerPreference, PresentMode, Queue, RequestAdapterError, RequestAdapterOptions,
    RequestDeviceError, Surface, SurfaceCapabilities, SurfaceColorSpace, SurfaceConfiguration,
    TextureFormat, TextureUsages, Trace,
};
use winit::{
    dpi::Size,
    error::OsError,
    event_loop::{ActiveEventLoop, OwnedDisplayHandle},
    window::Window,
};

use crate::AppSettings;

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
        settings: &AppSettings,
    ) -> Result<Self, Box<dyn Error>> {
        let window = Self::make_window(event_loop, settings.size, &settings.name)?;
        debug!("Window: {:?}", window);

        let owned_display_handle = event_loop.owned_display_handle();
        let instance = Self::make_instance(owned_display_handle);
        debug!("Instance: {:?}", instance);

        let surface = Self::make_surface(&instance, &window)?;
        debug!("Surface: {:?}", surface);

        let adapter = Self::make_adapter(&instance, &surface)?;
        debug!("Adapter: {:?}", adapter);

        let adapter_info = adapter.get_info();
        log::info!("[orbital] Selected adapter: {}", adapter_info.name);
        log::info!("[orbital]   backend:    {:?}", adapter_info.backend);
        log::info!("[orbital]   device type: {:?}", adapter_info.device_type);
        log::info!("[orbital]   vendor ID:  0x{:04X}", adapter_info.vendor);
        log::info!("[orbital]   device ID:  0x{:04X}", adapter_info.device);

        log::info!("[orbital] All available adapters:");
        let all_adapters = block_on(instance.enumerate_adapters(Backends::all()));
        for adapter in all_adapters {
            let info = adapter.get_info();
            log::info!(
                "[orbital]   - {} (backend: {:?}, type: {:?})",
                info.name,
                info.backend,
                info.device_type,
            );
        }

        let (device, queue) = Self::make_device_and_queue(&adapter)?;
        debug!("Device: {:?}", device);
        debug!("Queue: {:?}", queue);

        let ctx = Self {
            window,
            instance,
            adapter,
            device,
            queue,
            surface,
        };

        let surface_configuration = ctx.make_surface_configuration(settings.vsync_enabled);
        ctx.reconfigure_surface(&surface_configuration);

        Ok(ctx)
    }

    fn make_window<S: Into<String>>(
        event_loop: &ActiveEventLoop,
        inner_size: Size,
        title: S,
    ) -> Result<Window, OsError> {
        event_loop.create_window(
            Window::default_attributes()
                .with_active(true)
                .with_inner_size(inner_size)
                .with_title(title),
        )
    }

    fn make_instance(owned_display_handle: OwnedDisplayHandle) -> Instance {
        #[cfg(target_os = "windows")]
        unsafe {
            // VK_LAYER_AMD_switchable_graphics hangs vkEnumeratePhysicalDevices
            // on some AMD driver versions. Disable the implicit layer.
            std::env::set_var("DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1", "1");
        }

        Instance::new(InstanceDescriptor {
            backends: Backends::from_env().unwrap_or(Backends::all()),
            flags: InstanceFlags::from_build_config(),
            backend_options: BackendOptions::from_env_or_default(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            display: Some(Box::new(owned_display_handle)),
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
            apply_limit_buckets: false,
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

    fn make_view_formats(
        capabilities: &SurfaceCapabilities,
    ) -> (TextureFormat, Vec<TextureFormat>) {
        let first_format = capabilities
            .formats
            .first()
            .expect("There must be at least one surface format!");
        let srgb_format = first_format.add_srgb_suffix();

        let base = srgb_format.remove_srgb_suffix();
        let view_formats: Vec<TextureFormat> = capabilities
            .formats
            .iter()
            .filter(|f| f.remove_srgb_suffix() == base)
            .copied()
            .collect();

        (srgb_format, view_formats)
    }

    pub fn get_first_view_format(&self) -> TextureFormat {
        self.surface
            .get_configuration()
            .expect("Surface must be configured first!")
            .format
    }

    pub fn make_surface_configuration(&self, vsync: bool) -> SurfaceConfiguration {
        let capabilities = self.surface.get_capabilities(&self.adapter);

        let present_mode = match vsync {
            true => PresentMode::AutoVsync,
            false => PresentMode::AutoNoVsync,
        };

        let window_size = self.window.inner_size();

        let (srgb_format, view_formats) = Self::make_view_formats(&capabilities);

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
                color_space: SurfaceColorSpace::Auto,
            });

        default_config.desired_maximum_frame_latency = 2;
        default_config.present_mode = present_mode;
        default_config.alpha_mode = CompositeAlphaMode::Auto;
        default_config.format = srgb_format;
        default_config.usage = TextureUsages::RENDER_ATTACHMENT;
        default_config.view_formats = view_formats;
        default_config.width = window_size.width;
        default_config.height = window_size.height;
        default_config.desired_maximum_frame_latency = 2;

        default_config
    }

    pub fn current_surface_texture(&self) -> CurrentSurfaceTexture {
        self.surface().get_current_texture()
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