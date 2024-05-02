use wgpu::{BindGroupLayoutDescriptor, Face, FrontFace, PolygonMode, PrimitiveTopology};

use super::ShaderDescriptor;

#[derive(Debug)]
pub struct PipelineDescriptor<'a> {
    pub identifier: &'static str,
    pub shader_descriptor: ShaderDescriptor,
    pub bind_group_descriptors: Vec<BindGroupLayoutDescriptor<'a>>,
    pub primitive_topology: PrimitiveTopology,
    pub front_face_order: FrontFace,
    pub cull_mode: Option<Face>,
    pub polygon_mode: PolygonMode,
}

impl<'a> Default for PipelineDescriptor<'a> {
    fn default() -> Self {
        Self {
            identifier: "default",
            shader_descriptor: Default::default(),
            bind_group_descriptors: Default::default(),
            primitive_topology: Default::default(),
            front_face_order: Default::default(),
            cull_mode: Default::default(),
            polygon_mode: Default::default(),
        }
    }
}

impl<'a> PipelineDescriptor<'a> {
    pub fn default_with_shader(shader_descriptor: ShaderDescriptor) -> Self {
        Self {
            identifier: &format!("{} Pipeline", shader_descriptor.identifier),
            shader_descriptor,
            ..Default::default()
        }
    }
}
