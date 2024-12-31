use std::sync::Arc;

use wgpu::{Face, FrontFace, PolygonMode, PrimitiveTopology};

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

impl PipelineDescriptor {
    pub fn default_skybox() -> Self {
        Self {
            shader_descriptor: Arc::new(include_str!("../shader/skybox.wgsl")),
            bind_group_layouts: vec![
                Material::world_environment_pipeline_bind_group_layout(),
                Camera::pipeline_bind_group_layout(),
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

        default.polygon_mode = PolygonMode::Line;
        default.cull_mode = None;

        default
    }

    pub fn debug_bounding_box() -> Self {
        let mut default = Self::default();

        default.shader_descriptor = Arc::new(include_str!("../shader/debug_bounding_box.wgsl"));
        default.bind_group_layouts = vec![Camera::pipeline_bind_group_layout()];
        default.polygon_mode = PolygonMode::Line;
        default.cull_mode = None;
        default.include_complex_vertex_buffer_layout = false;
        default.include_simple_vertex_buffer_layout = true;
        default.include_instance_buffer_layout = true;

        default
    }
}

impl Default for PipelineDescriptor {
    /// Default is PBR
    fn default() -> Self {
        Self {
            shader_descriptor: Arc::new(include_str!("../shader/standard_pbr.wgsl")),
            bind_group_layouts: vec![
                Material::pbr_pipeline_bind_group_layout(),
                Camera::pipeline_bind_group_layout(),
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
