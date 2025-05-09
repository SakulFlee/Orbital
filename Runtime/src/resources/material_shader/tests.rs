use wgpu::TextureFormat;

use crate::{MaterialDescriptor, MaterialShader, MaterialShaderDescriptor};

#[test]
fn default_realization() {
    let (_adapter, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = MaterialShaderDescriptor::default();
    let _render_pipeline = MaterialShader::from_descriptor(&descriptor, None, &device, &queue)
        .expect("Failed turning default material shader descriptor into render pipeline!");
}

#[test]
fn realization_custom_texture_format() {
    let (_adapter, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = MaterialShaderDescriptor::default();
    let _render_pipeline = MaterialShader::from_descriptor(
        &descriptor,
        Some(TextureFormat::R8Unorm),
        &device,
        &queue,
    )
    .expect("Failed turning default material shader descriptor into render pipeline!");
}

#[test]
fn alias_material_descriptor() {
    let _ = MaterialDescriptor::default();
}
