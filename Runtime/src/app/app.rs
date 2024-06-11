use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};

pub trait App {
    fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
    where
        Self: Sized;

    fn update(&mut self)
    where
        Self: Sized;

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue)
    where
        Self: Sized;
}
