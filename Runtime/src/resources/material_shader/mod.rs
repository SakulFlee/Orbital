use std::error::Error;
use std::sync::OnceLock;

use wgpu::{
    BindGroup, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
    Device, FragmentState, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, TextureFormat, VertexState,
};

pub use crate::resources::shader::{ShaderDescriptor, ShaderError, Variables};

mod descriptor;
pub use descriptor::*;

mod vertex_stage_layout;
pub use vertex_stage_layout::*;

mod engine_bind_group_layout;
pub use engine_bind_group_layout::*;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MaterialShader {
    pipeline: RenderPipeline,
    bind_group: BindGroup,
    variables: Variables,
}

impl MaterialShader {
    /// Creates a realized `MaterialShader` from a given `MaterialShaderDescriptor`.
    /// If `surface_format` is set to `None`, the default value `TextureFormat::Rgba8UnormSrgb` will be used.
    /// All other arguments have to be supplied.
    ///
    /// `MaterialShaderDescriptor` supports `Default`!
    pub fn from_descriptor(
        descriptor: &MaterialShaderDescriptor,
        surface_format: Option<TextureFormat>,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Box<dyn Error>> {
        let shader_module = descriptor.shader_module(device)?;

        // Create a pipeline layout and bind group
        let (bind_group, layout, variables) = descriptor.bind_group(device, queue)?;

        let engine_bind_group_layout_once = OnceLock::new();
        let engine_bind_group_layout = engine_bind_group_layout_once
            .get_or_init(|| EngineBindGroupLayout::make_bind_group_layout(device));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: descriptor.name.as_deref(),
            bind_group_layouts: &[engine_bind_group_layout, &layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer_layouts = descriptor
            .vertex_stage_layouts
            .clone()
            .map(|vertex_stage_layouts| {
                vertex_stage_layouts
                    .clone()
                    .into_iter()
                    .map(|x| x.vertex_buffer_layout())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let depth_stencil = if descriptor.depth_stencil {
            Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            })
        } else {
            None
        };

        let targets = [Some(ColorTargetState {
            format: surface_format.unwrap_or(TextureFormat::Rgba8UnormSrgb),
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        // Create the actual render pipeline
        let label = descriptor.name();
        let pipeline_desc = RenderPipelineDescriptor {
            label: label.as_deref(),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some(descriptor.entrypoint_vertex),
                buffers: &vertex_buffer_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some(descriptor.entrypoint_fragment),
                targets: &targets,
                compilation_options: Default::default(),
            }),
            depth_stencil,
            primitive: PrimitiveState {
                topology: descriptor.primitive_topology,
                strip_index_format: None,
                front_face: descriptor.front_face_order,
                cull_mode: descriptor.cull_mode,
                unclipped_depth: false,
                polygon_mode: descriptor.polygon_mode,
                conservative: false,
            },
            cache: None,
            multiview: None,
            multisample: Default::default(),
        };

        let pipeline = device.create_render_pipeline(&pipeline_desc);
        Ok(Self {
            pipeline,
            bind_group,
            variables,
        })
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn variables(&self) -> &Variables {
        &self.variables
    }
}
