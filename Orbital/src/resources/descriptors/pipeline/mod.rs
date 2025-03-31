use std::sync::Arc;

use wgpu::{
    BindGroupDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Face, FrontFace, PolygonMode, PrimitiveTopology, ShaderStages,
};

use crate::{
    resources::realizations::{Camera, Material},
    world::PointLightStore,
};

use super::ShaderDescriptor;

mod bind_group_layout;
pub use bind_group_layout::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PipelineDescriptor {
    pub shader_descriptor: Arc<ShaderDescriptor>,
    pub bind_group_layouts: Vec<PipelineBindGroupLayout>,
    pub primitive_topology: PrimitiveTopology,
    pub front_face_order: FrontFace,
    pub cull_mode: Option<Face>,
    pub polygon_mode: PolygonMode,
    pub include_complex_vertex_buffer_layout: bool,
    pub include_simple_vertex_buffer_layout: bool,
    pub include_instance_buffer_layout: bool,
    pub depth_stencil: bool,
}

// TODO: Put this into a NEW descriptor and make pipeline in Material.
// TODO: Also move shader related stuff like entrypoint names into a new descriptor

impl PipelineDescriptor {
    pub fn wireframe_color_bind_group_layout_descriptor() -> PipelineBindGroupLayout {
        PipelineBindGroupLayout {
            label: Material::WIREFRAME_PIPELINE_BIND_GROUP_NAME,
            entries: vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }

    pub fn default_skybox() -> Self {
        Self {
            shader_descriptor: Arc::new(include_str!("../shader/skybox.wgsl")),
            bind_group_layouts: vec![
                Material::world_environment_pipeline_bind_group_layout(),
                Camera::bind_group_layout(),
            ],
            primitive_topology: Default::default(),
            front_face_order: FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            include_complex_vertex_buffer_layout: false,
            include_simple_vertex_buffer_layout: false,
            include_instance_buffer_layout: false,
            depth_stencil: false,
        }
    }

    pub fn default_wireframe() -> Self {
        let mut default = Self::default();

        // Set shader
        default.shader_descriptor = Arc::new(include_str!("../shader/wireframe.wgsl"));
        // Set bind group layouts, including special wireframe color one
        default.bind_group_layouts = vec![
            Camera::bind_group_layout(),
            Self::wireframe_color_bind_group_layout_descriptor(),
        ];
        // Set to line mode for wireframes
        default.polygon_mode = PolygonMode::Line;
        // Set simplified vertex layout and include buffer layout
        default.include_complex_vertex_buffer_layout = false;
        default.include_simple_vertex_buffer_layout = true;
        default.include_instance_buffer_layout = true;

        // default.cull_mode = None; // TODO: Check?

        default
    }
}

impl Default for PipelineDescriptor {
    /// Default is PBR
    // TODO: move to default_solid to match the rest above. Also decouple the above from using a default impl that might change.
    fn default() -> Self {
        Self {
            shader_descriptor: Arc::new(include_str!("../shader/standard_pbr.wgsl")),
            bind_group_layouts: vec![
                Material::pbr_pipeline_bind_group_layout(),
                Camera::bind_group_layout(),
                PointLightStore::pipeline_bind_group_layout(),
                Material::world_environment_pipeline_bind_group_layout(),
            ],
            primitive_topology: Default::default(),
            front_face_order: Default::default(),
            cull_mode: Some(Face::Back),
            polygon_mode: Default::default(),
            include_complex_vertex_buffer_layout: true,
            include_simple_vertex_buffer_layout: false,
            include_instance_buffer_layout: true,
            depth_stencil: true,
        }
    }
}

impl PipelineDescriptor {
    // Like `Default::default`, but with a custom shader
    pub fn default_with_shader(shader_descriptor: ShaderDescriptor) -> Self {
        Self {
            shader_descriptor: Arc::new(shader_descriptor),
            ..Default::default()
        }
    }
}
