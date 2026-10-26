use std::sync::Arc;

use orbital_shader_core::{ShaderDescriptor, VariableType};
use wgpu::{
    BlendState, CompareFunction, Face, FrontFace, PolygonMode, PrimitiveTopology, ShaderStages,
};

use crate::VertexStageLayout;

pub type MaterialDescriptor = MaterialShaderDescriptor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlphaMode {
    #[default]
    Opaque,
    /// Alpha-tested material. Stores the cutoff as `f32` bits so the descriptor
    /// remains `Eq + Hash` for the material cache.
    Mask(u32),
    Blend,
}

impl AlphaMode {
    pub fn mask(cutoff: f32) -> Self {
        Self::Mask(cutoff.to_bits())
    }

    pub fn cutoff(self) -> Option<f32> {
        match self {
            Self::Mask(bits) => Some(f32::from_bits(bits)),
            Self::Opaque | Self::Blend => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RenderQueue {
    #[default]
    Opaque,
    Transparent,
}

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
    pub alpha_mode: AlphaMode,
    pub blend: BlendState,
    pub depth_write: bool,
    pub depth_compare: CompareFunction,
    pub render_queue: RenderQueue,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
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
            alpha_mode: AlphaMode::Opaque,
            blend: BlendState::REPLACE,
            depth_write: true,
            depth_compare: CompareFunction::Less,
            render_queue: RenderQueue::Opaque,
            cast_shadows: true,
            receive_shadows: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_opaque_and_depth_tested() {
        let descriptor = MaterialShaderDescriptor::default();

        assert_eq!(descriptor.alpha_mode, AlphaMode::Opaque);
        assert_eq!(descriptor.render_queue, RenderQueue::Opaque);
        assert_eq!(descriptor.blend, BlendState::REPLACE);
        assert!(descriptor.depth_write);
        assert_eq!(descriptor.depth_compare, CompareFunction::Less);
        assert!(descriptor.cast_shadows);
        assert!(descriptor.receive_shadows);
    }

    #[test]
    fn alpha_mask_stores_cutoff_as_bits() {
        let mode = AlphaMode::mask(0.25);

        assert_eq!(mode.cutoff(), Some(0.25));
    }
}
