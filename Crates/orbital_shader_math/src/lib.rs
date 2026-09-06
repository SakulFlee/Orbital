//! # Shader Math Module
//!
//! A library of reusable, general-purpose WGSL math nodes (BRDF terms, tone
//! mapping, low-discrepancy sampling, cubemap helpers). Each node is a
//! [`orbital_shader_preprocessor::ShaderNode`] whose WGSL source lives in a
//! `.wgsl` file (via `include_str!`), making the library fully portable.

use orbital_shader_preprocessor::{NodeLibrary, ShaderNode};

/// Returns the general math node library. Register it with a
/// [`orbital_shader_preprocessor::NodeRegistry`] to make these nodes available
/// to shader builders by name.
pub fn math_library() -> NodeLibrary {
    let mut lib = NodeLibrary::new("orbital_shader_math");

    lib.add(ShaderNode::new("pi", include_str!("wgsl/pi.wgsl")));
    lib.add(ShaderNode::new(
        "f0_default",
        include_str!("wgsl/f0_default.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "aces_constants",
        include_str!("wgsl/aces_constants.wgsl"),
    ));
    lib.add(
        ShaderNode::new("aces_tone_map", include_str!("wgsl/aces_tone_map.wgsl"))
            .with_deps(["aces_constants"]),
    );
    lib.add(ShaderNode::new(
        "fresnel_schlick",
        include_str!("wgsl/fresnel_schlick.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "fresnel_schlick_roughness",
        include_str!("wgsl/fresnel_schlick_roughness.wgsl"),
    ));
    lib.add(
        ShaderNode::new(
            "distribution_ggx",
            include_str!("wgsl/distribution_ggx.wgsl"),
        )
        .with_deps(["pi"]),
    );
    lib.add(ShaderNode::new(
        "schlick_smith_ggx",
        include_str!("wgsl/schlick_smith_ggx.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "slope_scaled_bias",
        include_str!("wgsl/slope_scaled_bias.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "cubemap_face",
        include_str!("wgsl/cubemap_face.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "radical_inverse_vdc",
        include_str!("wgsl/radical_inverse_vdc.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "pcg_hash",
        include_str!("wgsl/pcg_hash.wgsl"),
    ));
    lib.add(
        ShaderNode::new(
            "hammersley2d_scrambled",
            include_str!("wgsl/hammersley2d_scrambled.wgsl"),
        )
        .with_deps(["radical_inverse_vdc"]),
    );
    lib.add(
        ShaderNode::new(
            "importance_sample_lambert",
            include_str!("wgsl/importance_sample_lambert.wgsl"),
        )
        .with_deps(["pi"]),
    );
    lib.add(ShaderNode::new(
        "radical_inverse",
        include_str!("wgsl/radical_inverse.wgsl"),
    ));
    lib.add(
        ShaderNode::new("hammersley", include_str!("wgsl/hammersley.wgsl"))
            .with_deps(["radical_inverse"]),
    );
    lib.add(
        ShaderNode::new(
            "importance_sample_ggx_ibl",
            include_str!("wgsl/importance_sample_ggx_ibl.wgsl"),
        )
        .with_deps(["pi"]),
    );
    lib.add(ShaderNode::new(
        "importance_sample_ggx_mip",
        include_str!("wgsl/importance_sample_ggx_mip.wgsl"),
    ));
    lib.add(ShaderNode::new(
        "luminance",
        include_str!("wgsl/luminance.wgsl"),
    ));
    lib.add(
        ShaderNode::new("fib_direction", include_str!("wgsl/fib_direction.wgsl")).with_deps(["pi"]),
    );
    lib.add(
        ShaderNode::new("disk_irradiance", include_str!("wgsl/disk_irradiance.wgsl"))
            .with_deps(["pi"]),
    );
    lib.add(ShaderNode::new(
        "star_hash",
        include_str!("wgsl/star_hash.wgsl"),
    ));

    lib
}
