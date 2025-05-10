use cgmath::Vector2;
use wgpu::{TextureFormat, TextureUsages};

use super::IblBrdf;

#[test]
fn test_empty() {
    let (_, device, _) = wgpu_test_adapter::make_wgpu_connection();

    const SIZE: u32 = 512;

    let result = IblBrdf::create_empty(
        Some("Test"),
        Vector2::new(SIZE, SIZE),
        TextureFormat::Rgba8UnormSrgb,
        TextureUsages::TEXTURE_BINDING,
        &device,
    );
    let _ = result
        .texture
        .expect("Texture must exist after generation!");
}

#[test]
fn test_generate() {
    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let result = IblBrdf::generate(&device, &queue);
    let _ = result
        .texture
        .expect("Texture must exist after generation!");
}
