use wgpu::{Adapter, Device, Queue, SurfaceConfiguration, TextureView};

pub trait App {
    fn init(
        config: &SurfaceConfiguration,
        adapter: &Adapter,
        device: &Device,
        queue: &Queue,
    ) -> Self
    where
        Self: Sized;

    fn resize(&mut self, config: &SurfaceConfiguration, device: &Device, queue: &Queue);

    fn update(&mut self);

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue);
}
