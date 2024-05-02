use hashbrown::HashMap;
use log::debug;
use ulid::Ulid;
use wgpu::{
    Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, StoreOp, TextureFormat, TextureView,
};

use crate::{resources::Model, runtime::Context};

pub struct RenderServer {
    models: Vec<Model>,
    model_preparation_queue: Vec<Model>,
    pipelines: HashMap<&'static str, RenderPipeline>,
    surface_texture_format: TextureFormat,
}

// TODO: Make descriptors work! :)

impl RenderServer {
    pub fn new(context: &Context, surface_texture_format: TextureFormat) -> Self {
        Self {
            models: Vec::new(),
            model_preparation_queue: Vec::new(),
            pipelines: HashMap::new(),
            surface_texture_format,
        }
    }

    pub fn set_surface_texture_format(&mut self, surface_texture_format: TextureFormat) {
        self.surface_texture_format = surface_texture_format;
    }

    pub fn add_model(&mut self, model: Model) {
        if self
            .pipelines
            .contains_key(model.material().render_pipeline_identifier())
        {
            // No preparation needed!
            // The model can be added directly to our models collection.
            self.models.push(model);
        } else {
            // Preparation is needed!
            // The model will be added to the preparation queue first.
            // After preparations are done, it will be added to the models
            // collection.
            self.model_preparation_queue.push(model);
        }
    }

    // TODO
    pub fn remove_mesh(&mut self, ulid: Ulid) {
        todo!()
        // if let Some(index) = self
        //     .meshes
        //     .iter()
        //     .enumerate()
        //     .find(|(_, e)| *e.ulid() == ulid)
        //     .map(|(i, _)| i)
        // {
        //     self.meshes.remove(index);
        // }
    }

    pub fn prepare(&mut self, context: &Context) {
        debug!("Preparation queue: {}", self.model_preparation_queue.len());

        while !self.model_preparation_queue.is_empty() {
            let model = self.model_preparation_queue.pop().unwrap();
            let pipeline_identifier = model.material().render_pipeline_identifier();
            let pipeline = model
                .material()
                .make_render_pipeline(context, Some(self.surface_texture_format));

            self.pipelines.insert(pipeline_identifier, pipeline);
            self.models.push(model);
        }
    }

    pub fn render(&mut self, view: &TextureView, context: &Context) {
        let mut encoder = context
            .device()
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
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

                let pipeline_identifier = material.render_pipeline_identifier();
                let pipeline = self
                    .pipelines
                    .get(&pipeline_identifier)
                    .expect(&format!("Pipeline '{}' is missing!", &pipeline_identifier));
                render_pass.set_pipeline(pipeline);

                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh.index_count(), 0, 0..1);
            }
        }

        context.queue().submit(Some(encoder.finish()));
    }
}
