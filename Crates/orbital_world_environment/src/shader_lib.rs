//! # World Environment Shader Library
//!
//! Reusable WGSL nodes specific to world environment material shaders.
//! These are assembled alongside engine/math library nodes by the
//! [`MaterialShaderDescriptor`] to produce the skybox and environment
//! material shaders.

use orbital_shader_preprocessor::{NodeLibrary, ShaderNode};

/// Returns the world-environment-specific node library.
///
/// Register it alongside the engine and math libraries with a
/// [`orbital_shader_preprocessor::NodeRegistry`]. The nodes here are the
/// **entrypoint bodies** for the two world-environment material shaders — the
/// shared struct definitions (`CameraUniform`, `Light`, `VertexOutput`, etc.)
/// come from the engine library.
pub fn world_environment_library() -> NodeLibrary {
    let mut lib = NodeLibrary::new("orbital_world_environment");

    // The texture-based skybox entrypoints (vertex + fragment).
    lib.add(
        ShaderNode::new(
            "world_env_texture_entrypoints",
            include_str!("wgsl_nodes/world_env_texture_entrypoints.wgsl"),
        )
        .with_deps([
            "camera_uniform",
            "light_struct",
            "light_types",
            "vertex_output",
            "aces_tone_map",
            "aces_constants",
        ]),
    );

    // The analytic skybox entrypoints (vertex + fragment). Uses `sky_color`
    // from the engine library instead of sampling a cube texture.
    lib.add(
        ShaderNode::new(
            "world_env_analytic_entrypoints",
            include_str!("wgsl_nodes/world_env_analytic_entrypoints.wgsl"),
        )
        .with_deps([
            "camera_uniform",
            "sky_params",
            "sky_color",
            "vertex_output",
            "aces_tone_map",
            "aces_constants",
        ]),
    );

    lib
}

/// Root node names for the texture-based world environment material shader.
/// Entrypoint bodies are provided via `raw_source`, not the node registry.
pub const WORLD_ENV_TEXTURE_NODES: &[&str] = &[
    // Engine structs
    "camera_uniform",
    "light_types",
    "light_struct",
    "vertex_output",
    // Math
    "aces_constants",
    "aces_tone_map",
];

/// Root node names for the analytic world environment material shader.
/// Entrypoint bodies are provided via `raw_source`, not the node registry.
pub const WORLD_ENV_ANALYTIC_NODES: &[&str] = &[
    // Engine structs
    "camera_uniform",
    "vertex_output",
    // Math
    "pi",
    "star_hash",
    "aces_constants",
    "aces_tone_map",
    // Sky
    "sky_params",
    "sky_color",
];
