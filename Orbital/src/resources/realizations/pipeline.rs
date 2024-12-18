use std::sync::Arc;
use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FragmentState, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    StencilState, TextureFormat, VertexState,
};

use crate::{
    cache::{Cache, CacheEntry},
    error::Error,
    resources::{
        descriptors::{PipelineDescriptor, ShaderDescriptor},
        realizations::Shader,
    },
};

use super::{Instance, Vertex};

#[derive(Debug)]
pub struct Pipeline {
    render_pipeline: RenderPipeline,
    bind_group_layouts: Vec<(&'static str, BindGroupLayout)>,
    shader: Arc<Shader>,
}

impl Pipeline {
    pub fn from_descriptor(
        pipeline_descriptor: &PipelineDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
        with_shader_cache: Option<&mut Cache<Arc<ShaderDescriptor>, Shader>>,
    ) -> Result<Pipeline, Error> {
        let bind_group_layouts = pipeline_descriptor
            .bind_group_layouts
            .iter()
            .map(|x| (x.label, x.make_bind_group_layout(device)))
            .collect::<Vec<_>>();
        let bind_group_layouts_ref = bind_group_layouts
            .iter()
            .map(|(_, x)| x)
            .collect::<Vec<_>>();

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts_ref,
            push_constant_ranges: &[],
        });

        let shader = if let Some(cache) = with_shader_cache {
            cache
                .entry(pipeline_descriptor.shader_descriptor.clone())
                .or_insert(CacheEntry::new(Shader::from_descriptor(
                    pipeline_descriptor.shader_descriptor.clone(),
                    device,
                    queue,
                )?))
                .clone_inner()
        } else {
            Arc::new(Shader::from_descriptor(
                pipeline_descriptor.shader_descriptor.clone(),
                device,
                queue,
            )?)
        };

        let mut vertex_buffers = Vec::new();
        if pipeline_descriptor.include_vertex_buffer_layout {
            vertex_buffers.push(Vertex::vertex_buffer_layout_descriptor());
        }
        if pipeline_descriptor.include_instance_buffer_layout {
            vertex_buffers.push(Instance::vertex_buffer_layout_descriptor());
        }

        let depth_stencil = if pipeline_descriptor.depth_stencil {
            Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            })
        } else {
            None
        };

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: shader.shader_module(),
                entry_point: Some("entrypoint_vertex"),
                buffers: &vertex_buffers,
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: shader.shader_module(),
                entry_point: Some("entrypoint_fragment"),
                targets: &[Some(ColorTargetState {
                    format: *surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
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
            depth_stencil,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            render_pipeline,
            bind_group_layouts,
            shader,
        })
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }

    pub fn bind_group_layout(&self, label: &str) -> Option<&BindGroupLayout> {
        self.bind_group_layouts
            .iter()
            .find(|x| x.0 == label)
            .map(|(_, x)| x)
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}
