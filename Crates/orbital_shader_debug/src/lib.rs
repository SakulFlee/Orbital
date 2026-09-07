//! # Debug/Overlay Shader Module
//!
//! Style shaders for debug/overlay rendering (e.g. wireframes). These assemble
//! full material shaders from the reusable engine/math node libraries plus a
//! small shader-specific entrypoint.

use orbital_material_shader::{MaterialShaderDescriptor, VertexStageLayout};
use orbital_shader_preprocessor::NodeLibrary;
use wgpu::{PolygonMode, PrimitiveTopology};

/// The root node names that assemble the wireframe shader.
pub const WIREFRAME_NODES: &[&str] = &[
    "camera_uniform",
    "vertex_data_simple",
    "instance_data",
    "fragment_data",
];

/// Builds a wireframe [`MaterialShaderDescriptor`].
pub fn wireframe_descriptor() -> MaterialShaderDescriptor {
    let mut base = MaterialShaderDescriptor::default();
    base.name = Some("Wireframe Material Shader".to_string());
    base.nodes = WIREFRAME_NODES;
    base.raw_source = Some(include_str!("wgsl/wireframe_entrypoints.wgsl").into());
    base.entrypoint_vertex = "entrypoint_vertex";
    base.entrypoint_fragment = "entrypoint_fragment";
    base.vertex_stage_layouts = Some(vec![
        VertexStageLayout::SimpleVertexData,
        VertexStageLayout::InstanceData,
    ]);
    base.primitive_topology = PrimitiveTopology::LineList;
    base.polygon_mode = PolygonMode::Fill;
    base.cull_mode = None;
    base.depth_stencil = true;
    base
}

/// Returns the debug shader node library (currently empty aside from any
/// future debug-specific nodes). Kept for API symmetry with other shader
/// crates; wireframe reuses the engine/math libraries.
pub fn debug_library() -> NodeLibrary {
    NodeLibrary::new("orbital_shader_debug")
}
