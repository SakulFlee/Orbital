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
    pub include_camera_bind_group_layout: bool,
    pub include_light_storage_bind_group_layout: bool,
    pub include_vertex_buffer_layout: bool,
    pub include_instance_buffer_layout: bool,
    pub depth_stencil: bool,
}

impl PipelineDescriptor {
    pub fn default_skybox() -> Self {
        Self {
            shader_descriptor: include_str!("shader/skybox.wgsl"),
            bind_group_entries: vec![
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
            primitive_topology: Default::default(),
            front_face_order: Default::default(),
            cull_mode: None,
            polygon_mode: Default::default(),
            include_camera_bind_group_layout: true,
            include_light_storage_bind_group_layout: false,
            include_vertex_buffer_layout: false,
            include_instance_buffer_layout: false,
            depth_stencil: false,
        }
    }
}

impl Default for PipelineDescriptor {
    /// Default is PBR
    fn default() -> Self {
        Self {
            shader_descriptor: include_str!("shader/standard_pbr.wgsl"),
            bind_group_entries: vec![
                // Normal
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
                // Albedo
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Metallic
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Roughness
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Occlusion
                BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 9,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            primitive_topology: Default::default(),
            front_face_order: Default::default(),
            cull_mode: Some(Face::Back),
            polygon_mode: Default::default(),
            include_camera_bind_group_layout: true,
            include_light_storage_bind_group_layout: true,
            include_vertex_buffer_layout: true,
            include_instance_buffer_layout: true,
            depth_stencil: true,
        }
    }
}

impl PipelineDescriptor {
    // TODO
    // Like `Default::default`, but with a custom shader
    pub fn default_with_shader(shader_descriptor: ShaderDescriptor) -> Self {
        Self {
            shader_descriptor,
            ..Default::default()
        }
    }
}
