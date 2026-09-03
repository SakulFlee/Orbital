use std::{error::Error, mem::transmute, sync::Arc};

use log::{debug, error, info, warn};
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
    /// The GPU surface. `None` while the app is suspended on Android (the
    /// native window was destroyed) and recreated on resume.
    surface: Option<Surface<'static>>,
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

        // Surface wgpu errors through the `log` crate so they appear in logcat
        // (`rust_std_out` tag) instead of a panic whose output gets filtered.
        device.on_uncaptured_error(Arc::new(|err| {
            error!("wgpu uncaptured error: {err:?}");
        }));

        let ctx = Self {
            window,
            instance,
            adapter,
            device,
            queue,
            surface: Some(surface),
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
        // Only request features the adapter actually reports. In particular
        // `POLYGON_MODE_LINE` is commonly missing on mobile GPUs (Android), and
        // requesting unsupported features makes `request_device` fail outright.
        let mut features = Features::default();
        for feature in [
            Features::POLYGON_MODE_LINE,
            Features::TIMESTAMP_QUERY,
            Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
        ] {
            if adapter.features().contains(feature) {
                features |= feature;
            }
        }

        block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("Orbital GPU"),
            required_features: features,
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
            experimental_features: ExperimentalFeatures::disabled(),
        }))
    }

    fn make_view_formats(
        capabilities: &SurfaceCapabilities,
    ) -> (TextureFormat, Vec<TextureFormat>) {
        // DIAG: Force linear Bgra8Unorm for tablet test
        let linear = capabilities
            .formats
            .iter()
            .find(|f| {
                !f.is_srgb() && matches!(f, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm)
            })
            .copied()
            .unwrap_or_else(|| capabilities.formats.first().copied().expect("No surface formats"));
        let srgb_format = linear; // Use linear for presentation on this device

        let base = srgb_format.remove_srgb_suffix();
        let view_formats: Vec<TextureFormat> = capabilities
            .formats
            .iter()
            .filter(|f| f.remove_srgb_suffix() == base)
            .copied()
            .collect();

        (srgb_format, view_formats)
    }

    pub fn adapter_features(&self) -> wgpu::Features {
        self.adapter.features()
    }

    pub fn get_first_view_format(&self) -> TextureFormat {
        self.surface
            .as_ref()
            .expect("Surface must be present (app not suspended)!")
            .get_configuration()
            .expect("Surface must be configured first!")
            .format
    }

    pub fn make_surface_configuration(&self, vsync: bool) -> SurfaceConfiguration {
        let surface = self
            .surface
            .as_ref()
            .expect("Surface must be present (app not suspended)!");
        let capabilities = surface.get_capabilities(&self.adapter);

        let present_mode = match vsync {
            true => PresentMode::AutoVsync,
            false => PresentMode::Immediate,
        };

        info!(
            "[Surface] Supported present modes: {:?}, selected: {:?} (vsync={})",
            capabilities.present_modes, present_mode, vsync
        );

        let window_size = self.window.inner_size();

        let (srgb_format, view_formats) = Self::make_view_formats(&capabilities);

        // Some adapters (e.g. the Android emulator's Vulkan backend) do not
        // support `SURFACE_VIEW_FORMATS`; configuring a surface with a
        // non-empty `view_formats` list fails with `MissingDownlevelFlags`.
        // Only request them when the adapter supports the flag.
        let supports_view_formats = self
            .adapter
            .get_downlevel_capabilities()
            .flags
            .contains(wgpu::DownlevelFlags::SURFACE_VIEW_FORMATS);
        if !supports_view_formats {
            warn!(
                "[Surface] SURFACE_VIEW_FORMATS not supported; configuring surface without view_formats"
            );
        }

        let mut default_config = surface
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
        default_config.view_formats = if supports_view_formats {
            view_formats
        } else {
            vec![]
        };
        default_config.width = window_size.width;
        default_config.height = window_size.height;
        default_config.desired_maximum_frame_latency = 2;

        // DIAG: Log final surface config to diagnose tablet black screen
        info!(
            "[Surface] Final config: format={:?} present_mode={:?} alpha_mode={:?} color_space={:?} view_formats={:?} size={}x{}",
            default_config.format, default_config.present_mode, default_config.alpha_mode,
            default_config.color_space, default_config.view_formats,
            default_config.width, default_config.height
        );

        default_config
    }

    pub fn current_surface_texture(&self) -> CurrentSurfaceTexture {
        self.surface().get_current_texture()
    }

    pub fn reconfigure_surface(&self, configuration: &SurfaceConfiguration) {
        let surface = self
            .surface
            .as_ref()
            .expect("Surface must be present (app not suspended)!");
        surface.configure(&self.device, configuration);
    }

    /// Drops the GPU surface. Called on suspend on Android, where the native
    /// window is destroyed; the surface must be recreated on resume.
    pub fn drop_surface(&mut self) {
        self.surface = None;
    }

    /// Recreates the GPU surface from the current (recreated) native window,
    /// reconfigures it, and returns the resulting surface configuration.
    /// Called on resume after [`AppContext::drop_surface`].
    pub fn recreate_surface(&mut self, vsync: bool) -> SurfaceConfiguration {
        let surface = Self::make_surface(&self.instance, &self.window)
            .expect("Failed to recreate surface on resume");
        self.surface = Some(surface);
        let config = self.make_surface_configuration(vsync);
        self.reconfigure_surface(&config);
        config
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
        self.surface
            .as_ref()
            .expect("Surface must be present (app not suspended)!")
    }

    pub fn surface_mut(&mut self) -> &mut Surface<'static> {
        self.surface
            .as_mut()
            .expect("Surface must be present (app not suspended)!")
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }
}
