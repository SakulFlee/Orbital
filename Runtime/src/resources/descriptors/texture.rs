use image::DynamicImage;
use wgpu::Color;

#[derive(Debug, Clone)]
pub enum TextureDescriptor {
    StandardSRGBu8Image(DynamicImage),
    StandardSRGBu8Data(Vec<u8>, Vector2<u32>),
    StandardSRGBu8Data(Vec<u8>, (u32, u32)),
    UniformColor(Color),
    Custom(
        wgpu::TextureDescriptor<'static>,
        wgpu::TextureViewDescriptor<'static>,
        wgpu::SamplerDescriptor<'static>,
    ),
}

impl TextureDescriptor {
    pub const EMPTY: Self = Self::UniformColor(Color::BLACK);
    pub const UNIFORM_BLACK: Self = Self::UniformColor(Color::BLACK);
    pub const UNIFORM_WHITE: Self = Self::UniformColor(Color::WHITE);
    pub const UNIFORM_GRAY: Self = Self::UniformColor(Color {
        r: 0.75,
        g: 0.75,
        b: 0.75,
        a: 1.0,
    });
}
