use std::sync::OnceLock;

use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Device, Face,
    FragmentState, FrontFace, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipelineDescriptor, ShaderStages, TextureFormat, VertexState,
};

use shader::{Error, ShaderDescriptor, ShaderSource, VariableType};

mod vertex_stage_layout;
pub use vertex_stage_layout::*;

mod engine_bind_group_layout;
pub use engine_bind_group_layout::*;

#[cfg(test)]
mod tests;

pub type MaterialShader = MaterialShaderDescriptor;
pub type MaterialDescriptor = MaterialShaderDescriptor;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MaterialShaderDescriptor {
    pub name: Option<&'static str>,
    pub shader_source: ShaderSource,
    pub variables: Vec<VariableType>,
    pub entrypoint_vertex: &'static str,
    pub entrypoint_fragment: &'static str,
    pub vertex_stage_layouts: Vec<VertexStageLayout>,
    pub primitive_topology: PrimitiveTopology,
    pub front_face_order: FrontFace,
    pub cull_mode: Option<Face>,
    pub polygon_mode: PolygonMode,
    pub depth_stencil: bool,
}

impl MaterialShaderDescriptor {
    fn create_render_pipeline(
        &self,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<wgpu::RenderPipeline, Error> {
        let shader_module = self.shader_module(device)?;

        // Create pipeline layout and bind group
        let (layout, variables) = self.bind_group_layout(device, queue)?;

        let engine_bind_group_layout_once = OnceLock::new();
        let engine_bind_group_layout = engine_bind_group_layout_once
            .get_or_init(|| EngineBindGroupLayout::make_bind_group_layout(device));

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: self.name,
            bind_group_layouts: &[&engine_bind_group_layout, &layout],
            push_constant_ranges: &[],
        });

        let vertex_buffer_layouts = self
            .vertex_stage_layouts
            .clone()
            .into_iter()
            .map(|x| x.vertex_buffer_layout())
            .collect::<Vec<_>>();

        let depth_stencil = if self.depth_stencil {
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
            format: *surface_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        // Create the actual render pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: self.name(),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some(self.entrypoint_vertex),
                buffers: &vertex_buffer_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader_module,
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
            multiview: None,
            multisample: Default::default(),
        };

        Ok(device.create_render_pipeline(&pipeline_desc))
    }
}

impl ShaderDescriptor for MaterialShaderDescriptor {
    fn name(&self) -> Option<&'static str> {
        self.name
    }

    fn source(&self) -> ShaderSource {
        self.shader_source
    }

    fn variables(&self) -> Option<Vec<VariableType>> {
        Some(self.variables.clone())
    }

    fn stages(&self) -> ShaderStages {
        ShaderStages::VERTEX_FRAGMENT
    }
}

impl Default for MaterialShaderDescriptor {
    fn default() -> Self {
        Self {
            name: Some("Default Material Shader"),
            shader_source: ShaderSource::default(),
            variables: Vec::new(),
            entrypoint_vertex: "entrypoint_vertex",
            entrypoint_fragment: "entrypoint_fragment",
            vertex_stage_layouts: vec![
                VertexStageLayout::SimpleVertexData,
                VertexStageLayout::InstanceData,
            ],
            primitive_topology: PrimitiveTopology::TriangleList,
            front_face_order: FrontFace::Ccw,
            cull_mode: Some(Face::Front),
            polygon_mode: PolygonMode::Fill,
            depth_stencil: true,
        }
    }
}
