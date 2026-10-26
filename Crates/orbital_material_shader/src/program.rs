//! Compiled shader program shared by material instances.
//!
//! A program owns the assembled WGSL module, the pipeline layout and the
//! material bind-group layout. It deliberately does not own textures, buffers
//! or a render pipeline: those are per material instance and per pass.

use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};

use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
    Device, Face, FragmentState, FrontFace, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, TextureFormat,
    VertexBufferLayout, VertexState,
};

use orbital_shader_core::ShaderDescriptor;

use crate::{MaterialShaderDescriptor, make_world_bind_group_layout};

static NEXT_PROGRAM_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub struct ShaderProgram {
    id: u64,
    name: Option<String>,
    module: wgpu::ShaderModule,
    pipeline_layout: PipelineLayout,
    material_bind_group_layout: Option<BindGroupLayout>,
    vertex_buffer_layouts: Vec<Option<VertexBufferLayout<'static>>>,
    entrypoint_vertex: &'static str,
    entrypoint_fragment: &'static str,
    primitive_topology: PrimitiveTopology,
    front_face_order: FrontFace,
    cull_mode: Option<Face>,
    polygon_mode: PolygonMode,
    depth_stencil: bool,
}

impl ShaderProgram {
    pub fn from_descriptor(
        descriptor: &MaterialShaderDescriptor,
        device: &Device,
    ) -> Result<Self, Box<dyn Error>> {
        let module = descriptor.shader_module(device)?;
        let material_bind_group_layout = descriptor.bind_group_layout_only(device);
        let world_bind_group_layout = make_world_bind_group_layout(device);

        let mut bind_group_layouts = vec![Some(&world_bind_group_layout)];
        if let Some(layout) = material_bind_group_layout.as_ref() {
            bind_group_layouts.push(Some(layout));
        }

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: descriptor.name.as_deref(),
            bind_group_layouts: &bind_group_layouts,
            immediate_size: 0,
        });

        let vertex_buffer_layouts = descriptor
            .vertex_stage_layouts
            .as_ref()
            .map(|layouts| {
                layouts
                    .iter()
                    .cloned()
                    .map(|layout| Some(layout.vertex_buffer_layout()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            id: NEXT_PROGRAM_ID.fetch_add(1, Ordering::Relaxed),
            name: descriptor.name.clone(),
            module,
            pipeline_layout,
            material_bind_group_layout,
            vertex_buffer_layouts,
            entrypoint_vertex: descriptor.entrypoint_vertex,
            entrypoint_fragment: descriptor.entrypoint_fragment,
            primitive_topology: descriptor.primitive_topology,
            front_face_order: descriptor.front_face_order,
            cull_mode: descriptor.cull_mode,
            polygon_mode: descriptor.polygon_mode,
            depth_stencil: descriptor.depth_stencil,
        })
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn material_bind_group_layout(&self) -> Option<&BindGroupLayout> {
        self.material_bind_group_layout.as_ref()
    }

    pub fn depth_format(&self) -> Option<TextureFormat> {
        self.depth_stencil.then_some(TextureFormat::Depth32Float)
    }

    pub fn create_forward_pipeline(
        &self,
        color_format: TextureFormat,
        device: &Device,
    ) -> RenderPipeline {
        let depth_stencil = self.depth_format().map(|format| DepthStencilState {
            format,
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        });

        let targets = [Some(ColorTargetState {
            format: color_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: self.name.as_deref(),
            layout: Some(&self.pipeline_layout),
            vertex: VertexState {
                module: &self.module,
                entry_point: Some(self.entrypoint_vertex),
                buffers: &self.vertex_buffer_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &self.module,
                entry_point: Some(self.entrypoint_fragment),
                targets: &targets,
                compilation_options: Default::default(),
            }),
            depth_stencil,
            primitive: PrimitiveState {
                topology: self.primitive_topology,
                strip_index_format: None,
                front_face: self.front_face_order,
                cull_mode: self.cull_mode,
                unclipped_depth: false,
                polygon_mode: self.polygon_mode,
                conservative: false,
            },
            cache: None,
            multisample: Default::default(),
            multiview_mask: None,
        })
    }
}
