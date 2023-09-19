use wgpu::{CompositeAlphaMode, PresentMode, SurfaceConfiguration, TextureFormat, TextureUsages};
use winit::window::Window;

pub trait SurfaceConfigurationHelper {
    fn from_window(
        surface_texture_format: TextureFormat,
        window: &Window,
        present_mode: PresentMode,
        alpha_mode: CompositeAlphaMode,
    ) -> Self;
}

impl SurfaceConfigurationHelper for SurfaceConfiguration {
    fn from_window(
        surface_texture_format: TextureFormat,
        window: &Window,
        present_mode: PresentMode,
        alpha_mode: CompositeAlphaMode,
    ) -> Self {
        Self {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_texture_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
        }
    }
}
