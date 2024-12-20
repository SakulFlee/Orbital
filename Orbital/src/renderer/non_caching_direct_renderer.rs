use async_trait::async_trait;
use cgmath::Vector2;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Device, IndexFormat, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    TextureFormat, TextureView,
};

use crate::error::Error;
use crate::log::error;
use crate::resources::realizations::{Material, Model};
use crate::resources::{descriptors::TextureDescriptor, realizations::Texture};
use crate::world::World;

use super::Renderer;

pub struct NonCachingDirectRenderer {
    surface_format: TextureFormat,
    depth_texture: Texture,
}

impl NonCachingDirectRenderer {
    fn render_skybox(
        &self,
        world: &World,
        encoder: &mut CommandEncoder,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // SkyBox
        let world_environment_material = match Material::from_descriptor(
            world.world_environment(),
            &self.surface_format,
            device,
            queue,
            None,
            None,
            None,
        ) {
            Ok(x) => x,
            Err(e) => {
                error!("SkyBox Material in invalid state: {:?}", e);
                return;
            }
        };

        render_pass.set_pipeline(world_environment_material.pipeline().render_pipeline());
        render_pass.set_bind_group(0, world_environment_material.bind_group(), &[]);
        render_pass.set_bind_group(1, world.active_camera().bind_group(), &[]);
        render_pass.draw(0..3, 0..1);
    }

    async fn render_models(
        &self,
        world: &World,
        encoder: &mut CommandEncoder,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
    ) -> Result<(), Error> {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
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

        // Models
        for model_descriptor in world.model_store().get_all() {
            let model = Model::from_descriptor(
                &*model_descriptor.read().await,
                &self.surface_format,
                device,
                queue,
                None,
                None,
                None,
                None,
                None,
            )?;

            render_pass.set_pipeline(model.material().pipeline().render_pipeline());

            render_pass.set_bind_group(0, model.material().bind_group(), &[]);
            render_pass.set_bind_group(1, world.active_camera().bind_group(), &[]);
            render_pass.set_bind_group(2, world.light_store().point_light_bind_group(), &[]);

            let world_environment_material = match Material::from_descriptor(
                world.world_environment(),
                &self.surface_format,
                device,
                queue,
                None,
                None,
                None,
            ) {
                Ok(x) => x,
                Err(e) => {
                    error!("WorldEnvironment Material in invalid state: {:?}", e);
                    return Err(e);
                }
            };
            render_pass.set_bind_group(3, world_environment_material.bind_group(), &[]);

            render_pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));
            render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
            render_pass
                .set_index_buffer(model.mesh().index_buffer().slice(..), IndexFormat::Uint32);

            render_pass.draw_indexed(0..model.mesh().index_count(), 0, 0..model.instance_count());
        }

        Ok(())
    }
}

#[async_trait]
impl Renderer for NonCachingDirectRenderer {
    fn new(
        surface_texture_format: wgpu::TextureFormat,
        resolution: cgmath::Vector2<u32>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        Self {
            surface_format: surface_texture_format,
            depth_texture: Texture::from_descriptor(
                &TextureDescriptor::Depth(resolution),
                device,
                queue,
            )
            .expect("Depth texture realization failed!"),
        }
    }

    async fn change_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        _device: &Device,
        _queue: &Queue,
    ) {
        // Set the format internally
        self.surface_format = surface_texture_format;
    }

    async fn change_resolution(
        &mut self,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) {
        // Remake the depth texture with the new size
        self.depth_texture = Texture::depth_texture(&resolution, device, queue);
    }

    async fn render(
        &mut self,
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
        world: &World,
    ) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            self.render_skybox(world, &mut encoder, target_view, device, queue);

            if let Err(e) = self
                .render_models(world, &mut encoder, target_view, device, queue)
                .await
            {
                error!("Failed to render models: {:?}", e);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
