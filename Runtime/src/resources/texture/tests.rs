use wgpu::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::{resources::{Texture, TextureDescriptor, TextureSize}, wgpu_test_adapter};

#[test]
fn test_data_descriptor_realization() {
    const SIZE: u32 = 64;

    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = TextureDescriptor::Data {
        pixels: (0..SIZE * SIZE)
            .into_iter()
            .map(|_| [0u8; 8])
            .flatten()
            .collect(),
        size: TextureSize {
            width: SIZE,
            height: SIZE,
            ..Default::default()
        },
        format: TextureFormat::Rgba8UnormSrgb,
        usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
    };

    let _texture =
        Texture::from_descriptor(&descriptor, &device, &queue).expect("Failure creating texture");
}

#[test]
fn test_custom_descriptor_realization() {
    const WIDTH: u32 = 64;
    const HEIGHT: u32 = 64;

    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = TextureDescriptor::Custom {
        texture_descriptor: wgpu::TextureDescriptor {
            label: Some("Test"),
            size: Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        view_descriptor: wgpu::TextureViewDescriptor {
            label: Some("Test"),
            ..Default::default()
        },
        sampler_descriptor: wgpu::SamplerDescriptor {
            label: Some("Test"),
            ..Default::default()
        },
        data: (0..WIDTH * HEIGHT).into_iter().map(|_| 0u8).collect(),
        size: Extent3d {
            width: WIDTH / 4,
            height: HEIGHT / 4,
            depth_or_array_layers: 1,
        },
    };

    let _texture =
        Texture::from_descriptor(&descriptor, &device, &queue).expect("Failure creating texture");
}
