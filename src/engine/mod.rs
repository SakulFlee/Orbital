use std::sync::Arc;

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
}

impl Engine {
    pub fn initialize(window: Arc<Window>) -> Self {
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
        let instance = Instance::new(InstanceDescriptor {
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
        let instance = Instance::new(InstanceDescriptor {
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
        let instance = Instance::new(InstanceDescriptor {
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
        let instance = Instance::new(InstanceDescriptor {
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
        let instance = Instance::new(InstanceDescriptor {
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
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        Self { window }
    }
}
