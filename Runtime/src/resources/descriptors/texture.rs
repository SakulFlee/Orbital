use image::DynamicImage;
use wgpu::{SamplerDescriptor, TextureViewDescriptor};

#[derive(Debug)]
pub enum TextureDescriptor<'a> {
    StandardSRGBu8Image(DynamicImage),
    StandardSRGBu8Data(&'a [u8], (u32, u32)),
    Custom(
        &'a wgpu::TextureDescriptor<'a>,
        &'a TextureViewDescriptor<'a>,
        &'a SamplerDescriptor<'a>,
    ),
}
