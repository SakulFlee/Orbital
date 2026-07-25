//! Validate the runtime-loaded WGSL shaders with naga (the same shader
//! frontend wgpu uses), so shader errors are caught by `cargo test`
//! instead of at application startup.

use std::path::Path;

fn validate(shader_path: &str) {
    let full = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(shader_path);
    let source = std::fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", full.display()));

    let module = naga::front::wgsl::parse_str(&source).unwrap_or_else(|e| {
        panic!(
            "{} failed WGSL parsing:\n{}",
            full.display(),
            e.emit_to_string(&source)
        )
    });

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator.validate(&module).unwrap_or_else(|e| {
        panic!(
            "{} failed WGSL validation:\n{}",
            full.display(),
            e.emit_to_string(&source)
        )
    });
}

#[test]
fn pbr_shader_is_valid_wgsl() {
    validate("Assets/Shaders/pbr.wgsl");
}

#[test]
fn shadow_depth_shader_is_valid_wgsl() {
    validate("Assets/Shaders/shadow_depth.wgsl");
}

#[test]
fn instance_cull_shader_is_valid_wgsl() {
    validate("Assets/Shaders/instance_cull.wgsl");
}
