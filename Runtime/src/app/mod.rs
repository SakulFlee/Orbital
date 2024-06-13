use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};

pub mod settings;
pub use settings::*;

pub mod runtime;
pub use runtime::*;

pub trait App {
    fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
    where
        Self: Sized;

    fn on_resize(&mut self, _new_size: cgmath::Vector2<u32>, _device: &Device, _queue: &Queue)
    where
        Self: Sized,
    {
        // Empty by default :)
    }

    fn on_update(&mut self)
    where
        Self: Sized;

    fn on_render(&mut self, view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized;
}
