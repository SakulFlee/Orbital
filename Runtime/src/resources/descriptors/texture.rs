use cgmath::Vector2;
use image::DynamicImage;
use wgpu::Color;

#[derive(Debug, Clone)]
pub enum TextureDescriptor {
    /// Creates a standard SRGB texture from a dynamic image.
    StandardSRGBu8Image(DynamicImage),
    /// Creates a standard SRGB texture from bytes (u8).
    /// Second parameter is the texture size.
    StandardSRGBu8Data(Vec<u8>, Vector2<u32>),
    /// Creates a texture with a single uniform color.
    UniformColor(Color),
    /// Creates a depth texture with a given size.
    Depth(Vector2<u32>),
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
