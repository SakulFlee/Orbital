use cgmath::Vector2;
use wgpu::{CommandBuffer, Device, Queue, TextureFormat, TextureView};

use crate::{resources::Camera, world::World};

use super::RenderEvent;

pub trait RenderSystem {
    async fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        device: &Device,
        queue: &Queue,
    );

    async fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue);

    async fn update(&mut self, world: &World, render_events: Option<Vec<RenderEvent>>);

    async fn render(
        &mut self,
        target_view: &TextureView,
        camera: &Camera,
        device: &Device,
        queue: &Queue,
    ) -> Option<CommandBuffer>;
}
