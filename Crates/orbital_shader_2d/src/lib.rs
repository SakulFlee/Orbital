//! # Shader 2D Module
//!
//! WGSL shader nodes for 2D rendering in the Orbital engine.
//!
//! Provides vertex and fragment shader building blocks for:
//! - 2D shape rendering (colored, textured)
//! - UI rendering
//! - Text rendering (SDF)

use orbital_shader_preprocessor::{NodeLibrary, ShaderNode};

/// Returns the 2D shader node library.
///
/// Register this library with the global node registry to make
/// 2D shader nodes available for shader assembly.
pub fn shader_2d_library() -> NodeLibrary {
    let mut lib = NodeLibrary::new("orbital_shader_2d");

    lib.add(ShaderNode::new(
        "vertex_2d_input",
        include_str!("wgsl/vertex_2d_input.wgsl"),
    ));

    lib.add(ShaderNode::new(
        "vertex_2d_output",
        include_str!("wgsl/vertex_2d_output.wgsl"),
    ));

    lib.add(ShaderNode::new(
        "vertex_2d_transform",
        include_str!("wgsl/vertex_2d_transform.wgsl"),
    ));

    lib.add(ShaderNode::new(
        "fragment_2d_color",
        include_str!("wgsl/fragment_2d_color.wgsl"),
    ));

    lib.add(ShaderNode::new(
        "fragment_2d_texture",
        include_str!("wgsl/fragment_2d_texture.wgsl"),
    ));

    lib
}
