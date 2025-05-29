use cgmath::Vector2;
use model::ModelRenderer;
use skybox::SkyBoxRenderer;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, CommandBuffer, CommandEncoderDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, LoadOp, Operations, PipelineLayoutDescriptor, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, ShaderModuleDescriptor, ShaderStages, StoreOp,
    TextureFormat, TextureView,
};

use crate::{
    resources::{Texture, WorldEnvironment},
    world::World,
};

mod frustum_check;
mod model;
mod skybox;
mod system;

mod event;
pub use event::*;

pub struct Renderer {
    surface_texture_format: TextureFormat,
    renderer_skybox: SkyBoxRenderer,
    renderer_model: ModelRenderer,
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
            renderer_skybox: SkyBoxRenderer::new(surface_texture_format, device, queue),
            renderer_model: ModelRenderer::new(depth_texture),
        }
    }

    pub fn set_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        _device: &Device,
        _queue: &Queue,
    ) {
        self.surface_texture_format = surface_texture_format;

        // TODO: maybe remake renderer?
    }

    pub fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue) {
        let depth_texture = Texture::depth_texture(&resolution, device, queue);

        self.renderer_model = ModelRenderer::new(depth_texture);

        // TODO: Skybox Renderer?
    }

    pub async fn update(&mut self, world: &World, render_events: Option<Vec<RenderEvent>>) {}

    pub async fn render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue) {
        todo!()
    }
}
