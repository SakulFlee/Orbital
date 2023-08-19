use std::sync::Arc;

use wgpu::{
    Adapter, Backends, CompositeAlphaMode, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PowerPreference, PresentMode, Queue, RequestAdapterOptions,
    Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
};

use crate::Window;

pub struct Engine {
    window: Arc<Window>,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl Engine {
    /// Initializes the `[Engine]`.
    /// Creates a bunch of critical internal components while doing so.
    pub async fn initialize(window: Arc<Window>) -> Self {
        let instance = Engine::make_instance().await;
        log::debug!("{instance:?}");

        let surface = Engine::make_surface(&instance, window.clone()).await;
        log::debug!("{surface:?}");

        let adapter = Engine::make_adapter(&instance, &surface).await;
        log::debug!("{adapter:?}");

        let (device, queue) = Engine::make_device(&adapter).await;
        log::debug!("{device:?}");
        log::debug!("{queue:?}");

        Self {
            window,
            surface,
            adapter,
            device,
            queue,
        }
    }

    /// Configures the local `[Surface]`.
    pub async fn configure_surface(&self) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Bgra8UnormSrgb,
                width: self.window.get_window().inner_size().width,
                height: self.window.get_window().inner_size().height,
                present_mode: PresentMode::Fifo,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![TextureFormat::Bgra8UnormSrgb],
            },
        )
    }

    /// Creates a new `[Device]` and, as a by-product a `[Queue]` of that `[Device]`.
    async fn make_device(adapter: &Adapter) -> (Device, Queue) {
        adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .expect("failed requesting device")
    }

    /// Creates a new `[Adapter]`.
    async fn make_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed requesting adapter")
    }

    /// Creates a new `[Surface]`.
    async fn make_surface(instance: &Instance, window: Arc<Window>) -> Surface {
        unsafe { instance.create_surface(&window.get_window()) }
            .expect("failed creating surface from window")
    }

    /// Creates an `[Instance]` given the graphics library.
    /// The `[Instance]` will automatically pick which graphics
    /// backend will be used.
    /// Currently supported are:
    /// - Vulkan
    /// - Metal
    /// - DX12
    /// - DX11
    /// - OpenGL (and WebGL)
    /// - WebGPU (virtual GPU used in Browsers)
    async fn make_instance() -> Instance {
        Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        })
    }

    /// Returns a new `[Arc]` of `[Window]`.
    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone()
    }

    /// Returns the local `[&Surface]`.
    pub fn get_surface(&self) -> &Surface {
        &self.surface
    }

    /// Returns the local `[&Adapter]`.
    pub fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    /// Returns the local `[&Device]`.
    pub fn get_device(&self) -> &Device {
        &self.device
    }

    /// Returns the local `[&Queue]`.
    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }
}
