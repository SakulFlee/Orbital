use hashbrown::HashMap;
use log::{debug, error};
use wgpu::{
    Color, CommandEncoderDescriptor, Device, IndexFormat, LoadOp, Operations, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

use crate::resources::{Model, ModelDescriptor, Pipeline};

pub struct RenderServer {
    models: Vec<Model>,
    models_to_spawn: Vec<ModelDescriptor>,
    pipelines: HashMap<String, Pipeline>,
    surface_texture_format: TextureFormat,
}

impl RenderServer {
    pub fn new(surface_texture_format: TextureFormat) -> Self {
        Self {
            models: Vec::new(),
            models_to_spawn: Vec::new(),
            pipelines: HashMap::new(),
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

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {
        debug!("Models to spawn: {}", self.models_to_spawn.len());

        while !self.models_to_spawn.is_empty() {
            let model_descriptor = self.models_to_spawn.pop().unwrap();

            let model = Model::from_descriptor(&model_descriptor, device, queue);

            if !self
                .pipelines
                .contains_key(&model.material().pipeline_descriptor().identifier as &str)
            {
                match Pipeline::from_descriptor(
                    model.material().pipeline_descriptor(),
                    self.surface_texture_format,
                    device,
                    queue,
                ) {
                    Ok(pipeline) => {
                        self.pipelines.insert(
                            model
                                .material()
                                .pipeline_descriptor()
                                .identifier
                                .to_string(),
                            pipeline,
                        );

                        self.models.push(model);
                    }
                    Err(e) => {
                        error!("Failed preparing pipeline '{}'! Skipping model preparation. Error: {:?}", &model.material().pipeline_descriptor().identifier, e);
                        continue;
                    }
                }
            }
        }
    }

    pub fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
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

                let pipeline_identifier = material.pipeline_descriptor().identifier.as_str();
                let pipeline = self
                    .pipelines
                    .get(pipeline_identifier)
                    .expect(&format!("Pipeline '{}' is missing!", &pipeline_identifier));
                render_pass.set_pipeline(pipeline.render_pipeline());

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh.index_count(), 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
    }
}
