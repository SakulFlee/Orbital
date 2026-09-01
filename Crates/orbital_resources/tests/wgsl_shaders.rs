//! Validate the runtime-loaded WGSL shaders with naga (the same shader
//! frontend wgpu uses), so shader errors are caught by `cargo test`
//! instead of at application startup.

use std::path::Path;

fn read_shader(shader_path: &str) -> String {
    let full = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(shader_path);
    std::fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", full.display()))
}

fn validate(label: &str, source: &str) {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|e| panic!("{label} failed WGSL parsing:\n{}", e.emit_to_string(source)));

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.validate(&module).unwrap_or_else(|e| {
        panic!(
            "{label} failed WGSL validation:\n{}",
            e.emit_to_string(source)
        )
    });
}

fn validate_single(shader_path: &str) {
    let source = read_shader(shader_path);
    validate(shader_path, &source);
}

fn validate_concatenated(shader_paths: &[&str]) {
    let label = shader_paths.join(" + ");
    let source = shader_paths
        .iter()
        .map(|p| read_shader(p))
        .collect::<Vec<_>>()
        .join("\n");
    validate(&label, &source);
}

#[test]
fn shadow_depth_shader_is_valid_wgsl() {
    validate_single("Assets/Shaders/shadow_depth.wgsl");
}

#[test]
fn sky_cube_shader_is_valid_wgsl() {
    validate_concatenated(&[
        "Crates/orbital_world_environment/src/sky_common.wgsl",
        "Crates/orbital_world_environment/src/generate_sky_cube.wgsl",
    ]);
}

#[test]
fn sky_diffuse_analytic_shader_is_valid_wgsl() {
    validate_concatenated(&[
        "Crates/orbital_world_environment/src/sky_common.wgsl",
        "Crates/orbital_world_environment/src/make_ibl_diffuse_analytic.wgsl",
    ]);
}

#[test]
fn sky_common_shader_is_valid_wgsl() {
    validate_single("Crates/orbital_world_environment/src/sky_common.wgsl");
}

#[test]
fn material_shader_analytic_is_valid_wgsl() {
    validate_concatenated(&[
        "Crates/orbital_world_environment/src/sky_common.wgsl",
        "Crates/orbital_world_environment/src/material_shader_analytic.wgsl",
    ]);
}

#[test]
fn material_shader_is_valid_wgsl() {
    validate_single("Crates/orbital_world_environment/src/material_shader.wgsl");
}

#[test]
fn make_ibl_diffuse_is_valid_wgsl() {
    validate_single("Crates/orbital_world_environment/src/make_ibl_diffuse.wgsl");
}

#[test]
fn make_ibl_specular_is_valid_wgsl() {
    validate_single("Crates/orbital_world_environment/src/make_ibl_specular.wgsl");
}

#[test]
fn make_mip_maps_is_valid_wgsl() {
    validate_single("Crates/orbital_world_environment/src/make_mip_maps.wgsl");
}

#[test]
fn generate_sky_cube_is_valid_wgsl() {
    validate_concatenated(&[
        "Crates/orbital_world_environment/src/sky_common.wgsl",
        "Crates/orbital_world_environment/src/generate_sky_cube.wgsl",
    ]);
}

#[test]
fn ibl_brdf_shader_is_valid_wgsl() {
    validate_single("Crates/orbital_ibl_brdf/src/shaders/ibl_brdf.wgsl");
}

// --- Node-graph shader validation -----------------------------------------
//
// The new node system assembles shaders from named, reusable WGSL nodes. These
// tests build representative shaders with the `ShaderBuilder` + prelude and
// validate the assembled output with naga, catching ordering/dedup mistakes.

fn validate_node_shader(label: &str, node_names: &[&str], raw: &str) {
    use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};
    use std::sync::Arc;

    let mut builder = ShaderBuilder::new(Arc::new(NodeRegistry::global().clone()));
    builder.add_nodes(node_names).unwrap();
    builder.add_source(raw);
    let source = builder.build();
    validate(label, &source);
}

fn validate_node_with_deps(label: &str, node_name: &str, extra: &str) {
    use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};
    use std::sync::Arc;

    let mut builder = ShaderBuilder::new(Arc::new(NodeRegistry::global().clone()));
    builder.add_node(node_name).unwrap();
    builder.add_source(extra);
    let source = builder.build();
    validate(label, &source);
}

#[test]
fn every_prelude_node_is_valid_with_its_dependencies() {
    use orbital_shader_preprocessor::prelude_library;

    let lib = prelude_library();

    let external_context: &[(&str, &str)] = &[];

    for node in &lib.nodes {
        let label = format!("prelude node '{}'", node.name);

        let extra = external_context
            .iter()
            .find(|(name, _)| *name == &*node.name)
            .map(|(_, src)| *src)
            .unwrap_or("");

        validate_node_with_deps(&label, &node.name, extra);
    }
}

#[test]
fn node_cubemap_face_shader_is_valid_wgsl() {
    let raw = r#"
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let face = gid_z_to_face(gid.z);
    let cube_uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(8.0) * 2.0 - 1.0;
    let dir = normalize(face.forward + face.right * cube_uv.x - face.up * cube_uv.y);
    var out = vec4<f32>(dir, 1.0);
}
"#;
    validate_node_shader("node cubemap face", &["cubemap_face"], raw);
}

#[test]
fn node_sky_math_shader_is_valid_wgsl() {
    let raw = r#"
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dir = fib_direction(gid.x, 64u);
    let n = vec3(0.0, 1.0, 0.0);
    let irr = disk_irradiance(dir, 0.01, vec3(1.0), n);
    let h = star_hash(gid.x, gid.y, gid.z);
    var out = vec4<f32>(irr, f32(h & 0xFFu) / 255.0);
}
"#;
    validate_node_shader(
        "node sky math",
        &["pi", "star_hash", "fib_direction", "disk_irradiance"],
        raw,
    );
}

#[test]
fn node_assembled_math_shader_is_valid_wgsl() {
    let raw = r#"
@fragment
fn entrypoint_fragment() -> @location(0) vec4<f32> {
    let f0 = vec3(F0_DEFAULT);
    let d = distribution_ggx(0.5, 0.3);
    let g = schlick_smith_ggx(0.8, 0.7, 0.3);
    let fr = fresnel_schlick_roughness(0.5, f0, 0.3);
    let color = aces_tone_map(vec3(d + g, fr.x, 0.0));
    return vec4<f32>(color, 1.0);
}
"#;
    validate_node_shader(
        "node math",
        &[
            "pi",
            "f0_default",
            "aces_tone_map",
            "distribution_ggx",
            "schlick_smith_ggx",
            "fresnel_schlick_roughness",
        ],
        raw,
    );
}

// --- PBR library node validation -------------------------------------------

fn validate_pbr_node_with_deps(label: &str, node_name: &str) {
    use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};
    use std::sync::Arc;

    let mut registry = NodeRegistry::new();
    let prelude = orbital_shader_preprocessor::prelude_library();
    registry.register_library(&prelude).unwrap();
    let pbr = orbital_shader_pbr::pbr_library();
    registry.register_library(&pbr).unwrap();

    let mut builder = ShaderBuilder::new(Arc::new(registry));
    builder.add_node(node_name).unwrap();
    let source = builder.build();
    validate(label, &source);
}

#[test]
fn every_pbr_node_is_valid_with_its_dependencies() {
    let lib = orbital_shader_pbr::pbr_library();

    // Nodes that reference runtime bindings (var<uniform>, var<storage>,
    // textureSample, etc.) cannot be validated in isolation — they need
    // binding declarations that are only present in the full entrypoint file.
    // Those nodes are covered by the `node_assembled_pbr_shader_is_valid_wgsl`
    // test below.
    let needs_bindings: &[&str] = &[
        "hdr_tone_map_gamma_correction",
        "sample_normal_from_map",
        "pbr_data",
        "sample_shadow_2d_pcf",
        "compute_shadow_for_light",
        "calculate_light_contribution",
    ];

    for node in &lib.nodes {
        if needs_bindings.contains(&&*node.name) {
            continue;
        }
        let label = format!("PBR node '{}'", node.name);
        validate_pbr_node_with_deps(&label, &node.name);
    }
}

// --- Full assembled PBR shader ---------------------------------------------

fn validate_assembled_pbr_shader(label: &str) {
    use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};
    use std::sync::Arc;

    let mut registry = NodeRegistry::new();
    let prelude = orbital_shader_preprocessor::prelude_library();
    registry.register_library(&prelude).unwrap();
    let pbr = orbital_shader_pbr::pbr_library();
    registry.register_library(&pbr).unwrap();

    let mut builder = ShaderBuilder::new(Arc::new(registry));
    for node_name in orbital_shader_pbr::PBR_NODES {
        builder.add_node(node_name).unwrap();
    }
    builder.add_source(include_str!(
        "../../../Crates/orbital_shader_pbr/src/wgsl/pbr_entrypoints.wgsl"
    ));
    let source = builder.build();
    validate(label, &source);
}

#[test]
fn node_assembled_pbr_shader_is_valid_wgsl() {
    validate_assembled_pbr_shader("assembled PBR shader");
}

// --- Full assembled world-environment shaders ------------------------------

fn validate_assembled_world_env(
    label: &str,
    node_names: &[&str],
    entrypoint_path: &str,
) {
    use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};
    use std::sync::Arc;

    let registry = Arc::new(NodeRegistry::global().clone());
    let mut builder = ShaderBuilder::new(registry);
    for node_name in node_names {
        builder.add_node(node_name).unwrap();
    }
    let entrypoint_src = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(entrypoint_path),
    )
    .unwrap_or_else(|e| panic!("failed to read {entrypoint_path}: {e}"));
    builder.add_source(&*entrypoint_src);
    let source = builder.build();
    validate(label, &source);
}

#[test]
fn node_assembled_world_env_texture_is_valid_wgsl() {
    validate_assembled_world_env(
        "assembled world-env texture shader",
        orbital_world_environment::WORLD_ENV_TEXTURE_NODES,
        "Crates/orbital_world_environment/src/wgsl_nodes/world_env_texture_entrypoints.wgsl",
    );
}

#[test]
fn node_assembled_world_env_analytic_is_valid_wgsl() {
    validate_assembled_world_env(
        "assembled world-env analytic shader",
        orbital_world_environment::WORLD_ENV_ANALYTIC_NODES,
        "Crates/orbital_world_environment/src/wgsl_nodes/world_env_analytic_entrypoints.wgsl",
    );
}
