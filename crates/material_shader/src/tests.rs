use wgpu::TextureFormat;

use crate::MaterialShaderDescriptor;

#[test]
fn default_realization() {
    let (_adapter, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = MaterialShaderDescriptor::default();
    let _render_pipeline = descriptor
        .create_render_pipeline(&TextureFormat::Rgba8UnormSrgb, &device, &queue)
        .expect("Failed turning default material shader descriptor into render pipeline!");
}
