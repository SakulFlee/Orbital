use log::info;
use ulid::Ulid;
use wgpu::{
    include_wgsl, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Face,
    FragmentState, FrontFace, IndexFormat, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    StoreOp, TextureFormat, TextureView, VertexState,
};

use crate::{
    resources::{Mesh, Resource, Vertex},
    runtime::Context,
};

pub struct RenderServer {
    meshes: Vec<Mesh>,
    render_pipeline: RenderPipeline,
}

impl RenderServer {
    pub fn new(context: &Context, surface_texture_format: TextureFormat) -> Self {
        Self {
            meshes: Vec::new(),
            render_pipeline: Self::make_render_pipeline(context, surface_texture_format),
        }
    }

    fn make_render_pipeline(
        context: &Context,
        surface_texture_format: TextureFormat,
    ) -> RenderPipeline {
        let shader = context
            .device()
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let layout = context
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        context
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "main_vs",
                    buffers: &[Vertex::descriptor()],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "main_fs",
                    targets: &[Some(ColorTargetState {
                        format: surface_texture_format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    pub fn remove_mesh(&mut self, ulid: Ulid) {
        if let Some(index) = self
            .meshes
            .iter()
            .enumerate()
            .find(|(_, e)| *e.ulid() == ulid)
            .map(|(i, _)| i)
        {
            self.meshes.remove(index);
        }
    }

    pub fn render(&self, view: &TextureView, context: &Context) {
        info!("Render Mesh Count: {}", self.meshes.len());

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

            render_pass.set_pipeline(&self.render_pipeline);

            for mesh in &self.meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));
                render_pass.set_index_buffer(mesh.index_buffer().slice(..), IndexFormat::Uint32);

                render_pass.draw_indexed(0..mesh.index_count(), 0, 0..1);
            }
        }

        context.queue().submit(Some(encoder.finish()));
    }
}
