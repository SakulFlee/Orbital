use std::sync::Arc;

use wgpu::{
    Adapter, CompositeAlphaMode, Device, DeviceDescriptor, Features, Limits, PowerPreference,
    PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages,
};
#[cfg(any(
    feature = "gl_vulkan",
    feature = "gl_metal",
    feature = "gl_dx12",
    feature = "gl_dx11",
    feature = "gl_opengl",
    feature = "gl_browser_webgpu",
))]
use wgpu::{Backends, Instance, InstanceDescriptor};

use crate::Window;

#[cfg(all(
    not(feature = "gl_vulkan"),
    not(feature = "gl_metal"),
    not(feature = "gl_dx12"),
    not(feature = "gl_dx11"),
    not(feature = "gl_opengl"),
    not(feature = "gl_browser_webgpu"),
))]
compile_error!("No graphics backend was selected! Check feature flags and recompile ...");

pub struct Engine {
    window: Arc<Window>,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl Engine {
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

    pub async fn configure(&self) {
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

    async fn make_surface(instance: &Instance, window: Arc<Window>) -> Surface {
        unsafe { instance.create_surface(&window.get_window()) }
            .expect("failed creating surface from window")
    }

    /// Creates an `[Instance]` given the graphics library.
    /// Valid graphics libraries are (feature flags!)
    /// - Vulkan (`gl_vulkan`)
    /// - Metal (`gl_metal`)
    /// - Dx12 (`gl_dx12`)
    /// - Dx11 (`gl_dx11`)
    /// - OpenGL (`gl_opengl`)
    /// - Browser WebGPU (`gl_browser_webgpu`)
    async fn make_instance() -> Instance {
        #[cfg(all(
            feature = "gl_vulkan",
            not(any(
                feature = "gl_metal",
                feature = "gl_dx12",
                feature = "gl_dx11",
                feature = "gl_opengl",
                feature = "gl_browser_webgpu",
            ))
        ))]
        return Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            ..Default::default()
        });
        #[cfg(all(
            feature = "gl_metal",
            not(any(
                feature = "gl_vulkan",
                feature = "gl_dx12",
                feature = "gl_dx11",
                feature = "gl_opengl",
                feature = "gl_browser_webgpu",
            ))
        ))]
        return Instance::new(InstanceDescriptor {
            backends: Backends::METAL,
            ..Default::default()
        });
        #[cfg(all(
            feature = "gl_dx12",
            not(any(
                feature = "gl_vulkan",
                feature = "gl_metal",
                feature = "gl_dx11",
                feature = "gl_opengl",
                feature = "gl_browser_webgpu",
            ))
        ))]
        return Instance::new(InstanceDescriptor {
            backends: Backends::DX12,
            ..Default::default()
        });
        #[cfg(all(
            feature = "gl_dx11",
            not(any(
                feature = "gl_vulkan",
                feature = "gl_metal",
                feature = "gl_dx12",
                feature = "gl_opengl",
                feature = "gl_browser_webgpu",
            ))
        ))]
        return Instance::new(InstanceDescriptor {
            backends: Backends::DX11,
            ..Default::default()
        });
        #[cfg(all(
            feature = "gl_opengl",
            not(any(
                feature = "gl_vulkan",
                feature = "gl_metal",
                feature = "gl_dx12",
                feature = "gl_dx11",
                feature = "gl_browser_webgpu",
            ))
        ))]
        return Instance::new(InstanceDescriptor {
            backends: Backends::GL,
            ..Default::default()
        });
        #[cfg(all(
            feature = "gl_browser_webgpu",
            not(any(
                feature = "gl_vulkan",
                feature = "gl_metal",
                feature = "gl_dx12",
                feature = "gl_dx11",
                feature = "gl_opengl",
            ))
        ))]
        return Instance::new(InstanceDescriptor {
            backends: Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
    }

    pub fn get_window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn get_surface(&self) -> &Surface {
        &self.surface
    }

    pub fn get_adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn get_device(&self) -> &Device {
        &self.device
    }

    pub fn get_queue(&self) -> &Queue {
        &self.queue
    }
}
