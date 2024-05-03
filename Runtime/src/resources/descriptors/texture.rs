use image::DynamicImage;

#[derive(Debug, Clone)]
pub enum TextureDescriptor {
    StandardSRGBu8Image(DynamicImage),
    StandardSRGBu8Data(Vec<u8>, (u32, u32)),
    Custom(
        wgpu::TextureDescriptor<'static>,
        wgpu::TextureViewDescriptor<'static>,
        wgpu::SamplerDescriptor<'static>,
    ),
}
