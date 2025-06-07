use cgmath::Vector2;
use log::debug;
use wgpu::{Device, Queue, TextureFormat, TextureView};

use crate::resources::{Model, Texture, WorldEnvironment};

pub struct Renderer {
    surface_texture_format: TextureFormat,
    depth_texture: Texture,
}

impl Renderer {
    pub fn new(
        surface_texture_format: TextureFormat,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let depth_texture = Texture::depth_texture(&resolution, device, queue);

        Self {
            surface_texture_format,
            depth_texture,
        }
    }

    pub fn set_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        _device: &Device,
        _queue: &Queue,
    ) {
        self.surface_texture_format = surface_texture_format;
    }

    pub fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue) {
        self.depth_texture = Texture::depth_texture(&resolution, device, queue);
    }

    pub async fn render(
        &mut self,
        target_view: &TextureView,
        world_environment: Option<&WorldEnvironment>,
        models: Vec<&Model>,
        device: &Device,
        queue: &Queue,
    ) {
        debug!("RENDER");
    }
}
