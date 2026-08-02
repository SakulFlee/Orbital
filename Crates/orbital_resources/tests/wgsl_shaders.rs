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
fn pbr_shader_is_valid_wgsl() {
    validate_single("Assets/Shaders/pbr.wgsl");
}

#[test]
fn shadow_depth_shader_is_valid_wgsl() {
    validate_single("Assets/Shaders/shadow_depth.wgsl");
}

#[test]
fn instance_cull_shader_is_valid_wgsl() {
    validate_single("Assets/Shaders/instance_cull.wgsl");
}

#[test]
fn sky_cube_shader_is_valid_wgsl() {
    validate_concatenated(&[
        "Crates/orbital_resources/src/world_environment/sky_common.wgsl",
        "Crates/orbital_resources/src/world_environment/generate_sky_cube.wgsl",
    ]);
}

#[test]
fn sky_diffuse_analytic_shader_is_valid_wgsl() {
    validate_concatenated(&[
        "Crates/orbital_resources/src/world_environment/sky_common.wgsl",
        "Crates/orbital_resources/src/world_environment/make_ibl_diffuse_analytic.wgsl",
    ]);
}
