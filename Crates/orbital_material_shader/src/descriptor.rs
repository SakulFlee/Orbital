use std::sync::Arc;

use orbital_shader_core::{ShaderDescriptor, VariableType};
use wgpu::{Face, FrontFace, PolygonMode, PrimitiveTopology, ShaderStages};

use crate::VertexStageLayout;

pub type MaterialDescriptor = MaterialShaderDescriptor;

/// Describes a full material shader in terms of the node-graph shader system.
///
/// A material shader is assembled from named [`orbital_shader_preprocessor::ShaderNode`]s
/// (see [`MaterialShaderDescriptor::nodes`]) resolved against the registry,
/// optionally followed by raw WGSL (entrypoints) via
/// [`MaterialShaderDescriptor::raw_source`].
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct MaterialShaderDescriptor {
    pub name: Option<String>,
    /// Root node names making up the shader (resolved against the registry).
    pub nodes: &'static [&'static str],
    /// Optional raw WGSL appended after the nodes (e.g. the vertex/fragment
    /// entrypoints). Owned so both `include_str!` and runtime-concatenated
    /// sources are supported.
    pub raw_source: Option<Arc<str>>,
    pub variables: Vec<VariableType>,
    pub entrypoint_vertex: &'static str,
    pub entrypoint_fragment: &'static str,
    pub vertex_stage_layouts: Option<Vec<VertexStageLayout>>,
    pub primitive_topology: PrimitiveTopology,
    pub front_face_order: FrontFace,
    pub cull_mode: Option<Face>,
    pub polygon_mode: PolygonMode,
    pub depth_stencil: bool,
}

impl ShaderDescriptor for MaterialShaderDescriptor {
    fn name(&self) -> Option<String> {
        self.name.clone()
    }

    fn nodes(&self) -> &[&str] {
        self.nodes
    }

    fn raw_source(&self) -> Option<Arc<str>> {
        self.raw_source.clone()
    }

    fn variables(&self) -> Option<&Vec<VariableType>> {
        Some(&self.variables)
    }

    fn stages(&self) -> ShaderStages {
        ShaderStages::VERTEX_FRAGMENT
    }
}

impl Default for MaterialShaderDescriptor {
    fn default() -> Self {
        Self {
            name: Some("Default Material Shader".to_string()),
            nodes: &[],
            raw_source: Some(Arc::from(include_str!("default_shader.wgsl"))),
            variables: Vec::new(),
            entrypoint_vertex: "entrypoint_vertex",
            entrypoint_fragment: "entrypoint_fragment",
            vertex_stage_layouts: Some(vec![
                VertexStageLayout::SimpleVertexData,
                VertexStageLayout::InstanceData,
            ]),
            primitive_topology: PrimitiveTopology::TriangleList,
            front_face_order: FrontFace::Ccw,
            cull_mode: Some(Face::Front),
            polygon_mode: PolygonMode::Fill,
            depth_stencil: true,
        }
    }
}
