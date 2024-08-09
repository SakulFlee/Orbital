use cgmath::Vector2;
use wgpu::{
    Color, CommandEncoderDescriptor, Device, IndexFormat, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    TextureFormat, TextureView,
};

use crate::{
    log::error,
    resources::{
        descriptors::TextureDescriptor,
        realizations::{Camera, LightStorage, Model, Pipeline, Texture},
    },
};

use super::Renderer;

pub struct StandardRenderer {
    surface_texture_format: TextureFormat,
    depth_texture: Texture,
}

impl Renderer for StandardRenderer {
    fn new(
        surface_texture_format: wgpu::TextureFormat,
        resolution: cgmath::Vector2<u32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            surface_texture_format,
            depth_texture: Texture::from_descriptor(
                &TextureDescriptor::Depth(resolution),
                device,
                queue,
            )
            .expect("Depth texture realization failed!"),
        }
    }

    fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        device: &Device,
        queue: &Queue,
    ) {
        // Set the format internally
        self.surface_texture_format = surface_texture_format;

        // The cache will automatically recompile itself
        // once a new format is used to access the cache.
        let _ = Pipeline::prepare_cache_access(Some(&surface_texture_format), device, queue);
    }

    fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue) {
        // Remake the depth texture with the new size
        self.depth_texture = Texture::depth_texture(&resolution, device, queue);
    }

    fn update(&mut self, _delta_time: f64) {}

    fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        models: &[&Model],
        camera: &Camera,
        light_storage: &LightStorage,
    ) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self.depth_texture.view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for model in models {
                let mesh = model.mesh();
                let material = match model.material(&self.surface_texture_format, device, queue) {
                    Ok(material) => material,
                    Err(e) => {
                        error!("Material failure: {:#?}", e);
                        error!("Skipping model render!");
                        continue;
                    }
                };

                let pipeline = match Pipeline::from_descriptor(
                    material.pipeline_descriptor(),
                    &self.surface_texture_format,
                    device,
                    queue,
                ) {
                    Ok(pipeline) => pipeline,
                    Err(e) => {
                        error!("Pipeline in invalid state! Error: {:?}", e);
                        continue;
                    }
                };

                render_pass.set_pipeline(pipeline.render_pipeline());

                render_pass.set_bind_group(0, material.bind_group(), &[]);
                render_pass.set_bind_group(1, camera.bind_group(), &[]);
                render_pass.set_bind_group(2, light_storage.bind_group().unwrap(), &[]);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint32);

                render_pass.draw_indexed(
                    0..mesh.index_count(),
                    0,
                    0..model.instances().len() as u32,
                );
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
