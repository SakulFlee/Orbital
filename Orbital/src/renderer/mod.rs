use async_trait::async_trait;
use cgmath::Vector2;
use wgpu::{Device, Queue, TextureFormat, TextureView};

use crate::world::World;

mod non_caching_direct_renderer;
pub use non_caching_direct_renderer::*;

mod caching_direct_renderer;
pub use caching_direct_renderer::*;

#[async_trait]
pub trait Renderer {
    fn new(
        surface_texture_format: TextureFormat,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,

        app_name: &str,
    ) -> Self;

    async fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        device: &Device,
        queue: &Queue,
    );

    async fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue);

    async fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        world: &World,
    );
}
