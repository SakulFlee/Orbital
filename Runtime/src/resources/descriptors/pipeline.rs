use wgpu::{
    BindGroupLayoutEntry, BindingType, Face, FrontFace, PolygonMode, PrimitiveTopology,
    SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

use super::ShaderDescriptor;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineDescriptor {
    pub shader_descriptor: ShaderDescriptor,
    pub bind_group_entries: Vec<BindGroupLayoutEntry>,
    pub primitive_topology: PrimitiveTopology,
    pub front_face_order: FrontFace,
    pub cull_mode: Option<Face>,
    pub polygon_mode: PolygonMode,
}

impl Default for PipelineDescriptor {
    /// Default is PBR
    fn default() -> Self {
        Self {
            shader_descriptor: include_str!("shader/standard_pbr.wgsl"), // TODO
            bind_group_entries: vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            primitive_topology: Default::default(),
            front_face_order: Default::default(),
            cull_mode: Default::default(),
            polygon_mode: Default::default(),
        }
    }
}

impl PipelineDescriptor {
    // Like `Default::default`, but with a custom shader
    pub fn default_with_shader(shader_descriptor: ShaderDescriptor) -> Self {
        Self {
            shader_descriptor,
            ..Default::default()
        }
    }
}
