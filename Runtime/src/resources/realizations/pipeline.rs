use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    TextureFormat, VertexState,
};

use crate::{
    error::Error,
    resources::{PipelineDescriptor, Shader, VertexUniform},
    runtime::Context,
};

pub struct Pipeline {
    identifier: String,
    render_pipeline: RenderPipeline,
    shader: Shader,
}

impl Pipeline {
    pub fn from_descriptor(
        pipeline_descriptor: &PipelineDescriptor,
        surface_format: TextureFormat,
        context: &Context,
    ) -> Result<Self, Error> {
        let mut bind_group_layouts = Vec::<BindGroupLayout>::new();
        for bind_group_layout_descriptor in &pipeline_descriptor.bind_group_descriptors {
            bind_group_layouts.push(
                context
                    .device()
                    .create_bind_group_layout(&bind_group_layout_descriptor),
            );
        }

        let render_pipeline_layout =
            context
                .device()
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&format!(
                        "{} Pipeline Layout",
                        pipeline_descriptor.identifier
                    )),
                    bind_group_layouts: bind_group_layouts
                        .iter()
                        .collect::<Vec<&BindGroupLayout>>()
                        .as_slice(),
                    push_constant_ranges: &[],
                });

        let shader = Shader::from_descriptor(&pipeline_descriptor.shader_descriptor, context)?;

        let pipeline = context
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(&pipeline_descriptor.identifier),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: shader.shader_module(),
                    entry_point: "entrypoint_vertex",
                    buffers: &[VertexUniform::descriptor()],
                },
                fragment: Some(FragmentState {
                    module: shader.shader_module(),
                    entry_point: "entrypoint_fragment",
                    targets: &[Some(ColorTargetState {
                        format: surface_format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: pipeline_descriptor.primitive_topology,
                    strip_index_format: None,
                    front_face: pipeline_descriptor.front_face_order,
                    cull_mode: pipeline_descriptor.cull_mode,
                    unclipped_depth: false,
                    polygon_mode: pipeline_descriptor.polygon_mode,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
            });

        Ok(Self {
            identifier: pipeline_descriptor.identifier.clone(),
            render_pipeline: pipeline,
            shader,
        })
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}
