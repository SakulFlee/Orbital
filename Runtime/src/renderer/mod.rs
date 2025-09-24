use cgmath::Vector2;
use wgpu::{
    BindGroup, Color, CommandEncoder, CommandEncoderDescriptor, Device, IndexFormat, LoadOp,
    Operations, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

use crate::resources::{MaterialShader, Model, Texture, WorldEnvironment};

pub struct Renderer {
    surface_texture_format: TextureFormat,
    depth_texture: Texture,
}

impl Renderer {
    pub fn surface_texture_format(&self) -> &TextureFormat {
        &self.surface_texture_format
    }
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
        world_bind_group: &BindGroup,
        world_environment_option: Option<&WorldEnvironment>,
        models: Vec<&Model>,
        device: &Device,
        queue: &Queue,
    ) {
        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Orbital::Render::Encoder"),
        });

        if let Some(world_environment) = world_environment_option {
            let sky_box_shader = world_environment.material_shader();
            self.render_sky_box(
                target_view,
                sky_box_shader,
                world_bind_group,
                &mut command_encoder,
            );
        }

        self.render_models(models, target_view, world_bind_group, &mut command_encoder);

        queue.submit(vec![command_encoder.finish()]);
    }

    fn render_sky_box(
        &self,
        target_view: &TextureView,
        sky_box_shader: &MaterialShader,
        world_bind_group: &BindGroup,
        command_encoder: &mut CommandEncoder,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::SkyBox"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(sky_box_shader.pipeline());

        render_pass.set_bind_group(0, world_bind_group, &[]);

        render_pass.draw(0..3, 0..1);
    }

    fn render_models(
        &self,
        models: Vec<&Model>,
        target_view: &TextureView,
        world_bind_group: &BindGroup,
        command_encoder: &mut CommandEncoder,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Model RenderPass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
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
            for material in model.materials() {
                render_pass.set_pipeline(material.pipeline());

                render_pass.set_bind_group(0, world_bind_group, &[]);
                render_pass.set_bind_group(1, material.bind_group(), &[]);

                render_pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));
                render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
                render_pass
                    .set_index_buffer(model.mesh().index_buffer().slice(..), IndexFormat::Uint32);

                render_pass.draw_indexed(
                    0..model.mesh().index_count(),
                    0,
                    0..model.instance_count(),
                );
            }
        }
    }
}
