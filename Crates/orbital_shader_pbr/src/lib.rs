//! # PBR Shader Module
//!
//! PBR-specific WGSL nodes (BRDF orchestration, material data gathering, IBL,
//! shadow sampling) plus the [`PBRMaterialShaderDescriptor`] that assembles a
//! full PBR material shader from the reusable node libraries.

use orbital_shader_preprocessor::{NodeLibrary, ShaderNode};

mod pbr_material_shader;
pub use pbr_material_shader::*;

/// Returns the PBR node library. Register it (alongside
/// [`orbital_shader_math::math_library`] and
/// [`orbital_shader_engine::engine_library`]) with a
/// [`orbital_shader_preprocessor::NodeRegistry`] to make these nodes available
/// to shader builders by name.
pub fn pbr_library() -> NodeLibrary {
    let mut lib = NodeLibrary::new("orbital_shader_pbr");

    // Structs
    lib.add(ShaderNode::new(
        "pbr_factors",
        include_str!("wgsl/pbr_factors.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "light_contribution",
        include_str!("wgsl/light_contribution.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "pbr_data_struct",
        include_str!("wgsl/pbr_data_struct.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "spot_bias_scale",
        include_str!("wgsl/spot_bias_scale.wgsl"),
    ));

    // Functions
    lib.add(
        ShaderNode::new(
            "hdr_tone_map_gamma_correction",
            include_str!("wgsl/hdr_tone_map_gamma_correction.wgsl"),
        )
        .with_deps(["camera_uniform"]),
    );
    lib.add(
        ShaderNode::new(
            "sample_normal_from_map",
            include_str!("wgsl/sample_normal_from_map.wgsl"),
        )
        .with_deps(["fragment_data"]),
    );
    lib.add(
        ShaderNode::new("pbr_data", include_str!("wgsl/pbr_data.wgsl")).with_deps([
            "pbr_data_struct",
            "pbr_factors",
            "sample_normal_from_map",
            "camera_uniform",
            "fragment_data",
        ]),
    );
    lib.add(
        ShaderNode::new(
            "calculate_light_brdf",
            include_str!("wgsl/calculate_light_brdf.wgsl"),
        )
        .with_deps([
            "light_contribution",
            "pbr_data_struct",
            "light_struct",
            "light_types",
            "distribution_ggx",
            "schlick_smith_ggx",
            "fresnel_schlick",
            "f0_default",
            "pi",
        ]),
    );
    lib.add(
        ShaderNode::new(
            "calculate_ambient_ibl",
            include_str!("wgsl/calculate_ambient_ibl.wgsl"),
        )
        .with_deps([
            "pbr_data_struct",
            "fresnel_schlick_roughness",
            "f0_default",
            "ambient_intensity",
        ]),
    );
    lib.add(
        ShaderNode::new(
            "sample_shadow_2d_pcf",
            include_str!("wgsl/sample_shadow_2d_pcf.wgsl"),
        )
        .with_deps(["shadow_slot"]),
    );
    lib.add(
        ShaderNode::new(
            "compute_shadow_for_light",
            include_str!("wgsl/compute_shadow_for_light.wgsl"),
        )
        .with_deps([
            "light_struct",
            "light_types",
            "shadow_types",
            "shadow_slot",
            "slope_scaled_bias",
            "sample_shadow_2d_pcf",
            "spot_bias_scale",
        ]),
    );
    lib.add(
        ShaderNode::new(
            "calculate_light_contribution",
            include_str!("wgsl/calculate_light_contribution.wgsl"),
        )
        .with_deps([
            "pbr_data_struct",
            "calculate_light_brdf",
            "compute_shadow_for_light",
        ]),
    );

    lib
}
