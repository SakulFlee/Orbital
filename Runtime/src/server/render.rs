use cgmath::Vector2;
use log::{debug, error};
use wgpu::{
    Color, CommandEncoderDescriptor, Device, IndexFormat, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    TextureFormat, TextureView,
};

use crate::resources::{
    descriptors::{CameraDescriptor, ModelDescriptor},
    realizations::{Camera, Model, Pipeline, Texture},
};

pub struct RenderServer {
    models: Vec<Model>,
    models_to_spawn: Vec<ModelDescriptor>,
    surface_texture_format: TextureFormat,
    depth_texture: Texture,
    camera: Camera,
}

impl RenderServer {
    pub fn new(
        surface_texture_format: TextureFormat,
        depth_texture_resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        Self {
            models: Vec::new(),
            models_to_spawn: Vec::new(),
            surface_texture_format,
            depth_texture: Texture::depth_texture(&depth_texture_resolution, device, queue),
            camera: Camera::from_descriptor(CameraDescriptor::default(), device, queue),
        }
    }

    pub fn change_depth_texture_resolution(
        &mut self,
        new_resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) {
        self.depth_texture = Texture::depth_texture(&new_resolution, device, queue);
    }

    pub fn set_surface_texture_format(&mut self, surface_texture_format: TextureFormat) {
        self.surface_texture_format = surface_texture_format;
    }

    pub fn spawn_model(&mut self, model_descriptor: ModelDescriptor) {
        self.models_to_spawn.push(model_descriptor);
    }

    pub fn despawn_model(&mut self) {
        todo!()
    }

    pub fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        self.do_prepare(device, queue);
        self.do_render(device, queue, view);
    }

    fn do_render(&mut self, device: &Device, queue: &Queue, view: &TextureView) {
        // TODO: Remove
        unsafe {
            static mut INCREMENT: bool = true;
            let mut x = *self.camera.descriptor();
            if INCREMENT {
                x.position.x += 0.001;
            } else {
                x.position.x -= 0.001;
            }

            if x.position.x > -1.0 {
                INCREMENT = false;
            }
            if x.position.x < -4.0 {
                INCREMENT = true;
            }
            self.camera.update_from_descriptor(x, device, queue);
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
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

            for model in &self.models {
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
                render_pass.set_bind_group(1, self.camera.bind_group(), &[]);

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

    pub fn do_prepare(&mut self, device: &Device, queue: &Queue) {
        if self.models_to_spawn.is_empty() {
            return;
        }
        debug!("Models to realize: {}", self.models_to_spawn.len());

        while let Some(model_descriptor) = self.models_to_spawn.pop() {
            match Model::from_descriptor(&model_descriptor, device, queue) {
                Ok(model) => self.models.push(model),
                Err(e) => {
                    error!(
                        "Failed preparing model! Skipping model preparation. Error: {:?}",
                        e
                    );
                    continue;
                }
            }
        }
    }
}
