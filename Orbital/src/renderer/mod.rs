use async_trait::async_trait;
use cgmath::Vector2;
use log::warn;
use wgpu::{Device, Queue, TextureFormat, TextureView};

use crate::world::{Message, World};

mod draw_indirect;
pub use draw_indirect::*;

mod draw_indexed_indirect;
pub use draw_indexed_indirect::*;

mod non_caching_direct_renderer;
pub use non_caching_direct_renderer::*;

mod caching_direct_renderer;
pub use caching_direct_renderer::*;

mod caching_indirect_renderer;
pub use caching_indirect_renderer::*;

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

    async fn on_message(&mut self, message: Message) {
        warn!(
            "Message received which isn't handled by the renderer. Message: {:?}",
            message
        );
    }

    async fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        world: &World,
    );
}
