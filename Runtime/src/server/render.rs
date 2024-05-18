use log::{debug, error};
use wgpu::{
    Color, CommandEncoderDescriptor, Device, IndexFormat, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

use crate::resources::{Model, ModelDescriptor, Pipeline};

// TODO: Material cache
// TODO: Mesh cache

pub struct RenderServer {
    models: Vec<Model>,
    models_to_spawn: Vec<ModelDescriptor>,
    surface_texture_format: TextureFormat,
}

impl RenderServer {
    pub fn new(surface_texture_format: TextureFormat) -> Self {
        Self {
            models: Vec::new(),
            models_to_spawn: Vec::new(),
            surface_texture_format,
        }
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
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for model in &self.models {
                let mesh = model.mesh();
                let material = model.material();

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

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh.index_count(), 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }

    pub fn do_prepare(&mut self, device: &Device, queue: &Queue) {
        if self.models_to_spawn.is_empty() {
            return;
        }
        debug!("Models to realize: {}", self.models_to_spawn.len());

        while !self.models_to_spawn.is_empty() {
            let model_descriptor = self.models_to_spawn.pop().unwrap();

            match Model::from_descriptor(
                &model_descriptor,
                &self.surface_texture_format,
                device,
                queue,
            ) {
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
