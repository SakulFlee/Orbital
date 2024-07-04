use cgmath::Vector2;
use wgpu::{Device, Queue, TextureFormat, TextureView};

use crate::resources::realizations::Model;

pub mod standard;
pub use standard::*;

pub mod test;
pub use test::*;

pub trait Renderer {
    fn new(
        surface_texture_format: TextureFormat,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self;

    fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        device: &Device,
        queue: &Queue,
    );

    fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue);

    fn update(&mut self, delta_time: f64);

    fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        models: &[&Model],
    );
}
