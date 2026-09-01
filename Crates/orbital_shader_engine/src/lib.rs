//! # Shader Engine Module
//!
//! A library of engine-wide WGSL nodes: shared uniform structs (camera, light,
//! shadow, sky) and the engine's vertex/instance/fragment layouts. These are
//! the "bindings/structs" that all engine material and environment shaders
//! share. The Rust bind-group layout builder lives in `orbital_resources`.

use orbital_shader_preprocessor::{NodeLibrary, ShaderNode};

/// Returns the engine-wide node library. Register it (alongside
/// [`orbital_shader_math::math_library`]) with a
/// [`orbital_shader_preprocessor::NodeRegistry`] to make these nodes available
/// to shader builders by name.
pub fn engine_library() -> NodeLibrary {
    let mut lib = NodeLibrary::new("orbital_shader_engine");

    lib.add(ShaderNode::new(
        "camera_uniform",
        include_str!("wgsl/camera_uniform.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "light_types",
        include_str!("wgsl/light_types.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "light_struct",
        include_str!("wgsl/light_struct.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "shadow_types",
        include_str!("wgsl/shadow_types.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "shadow_slot",
        include_str!("wgsl/shadow_slot.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "vertex_data_simple",
        include_str!("wgsl/vertex_data_simple.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "vertex_data_complex",
        include_str!("wgsl/vertex_data_complex.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "instance_data",
        include_str!("wgsl/instance_data.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "fragment_data",
        include_str!("wgsl/fragment_data.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "vertex_output",
        include_str!("wgsl/vertex_output.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "ambient_intensity",
        include_str!("wgsl/ambient_intensity.wgsl"),
    ));
    lib.add(ShaderNode::new("inv_atan", include_str!("wgsl/inv_atan.wgsl")));
    lib.add(ShaderNode::new(
        "tile_params",
        include_str!("wgsl/tile_params.wgsl"),
    ));
    lib.add(ShaderNode::new("mip_info", include_str!("wgsl/mip_info.wgsl")));
    lib.add(ShaderNode::new(
        "sky_params",
        include_str!("wgsl/sky_params.wgsl"),
    ));
    lib.add(
        ShaderNode::new("sky_color", include_str!("wgsl/sky_color.wgsl"))
            .with_deps(["sky_params", "star_hash"]),
    );

    lib
}
