use std::{ffi::OsString, hash::Hash};

use crate::resources::texture::filter_mode::FilterMode;
use crate::resources::TextureSize;
use wgpu::{Color, Extent3d, TextureDimension, TextureFormat, TextureUsages, TextureViewDimension};

#[derive(Debug, Clone, PartialEq)]
pub enum TextureDescriptor {
    /// Defines a path for a texture to be loaded from.
    /// ⚠️ This file has to be accessible @runtime!
    ///
    /// For supported formats check the [Image documentation](https://github.com/image-rs/image/blob/main/README.md#supported-image-formats).
    File {
        path: OsString,
        usages: TextureUsages,
    },
    /// Defines a texture directly from data.
    /// Assumes to be SRGBA-like data.
    ///
    /// The channels field defines how many channels are actually used.  
    /// `[TextureChannel::RGBA]` would result in Rgba8UnormSrgb.
    /// `[TextureChannel::R]` would result in R8Unorm.
    Data {
        pixels: Vec<u8>,
        size: TextureSize,
        usages: TextureUsages,
        format: TextureFormat,
        texture_dimension: TextureDimension,
        texture_view_dimension: TextureViewDimension,
        filter_mode: FilterMode,
    },
    /// In case you need a custom set of descriptors.
    Custom {
        /// Texture Descriptor.
        /// Check `wgpu::TextureDescriptor` for more information.
        texture_descriptor: wgpu::TextureDescriptor<'static>,
        /// Texture View Descriptor.
        /// Check `wgpu::TextureViewDescriptor` for more information.
        view_descriptor: wgpu::TextureViewDescriptor<'static>,
        /// Texture Sampler Descriptor.
        /// Check `wgpu::SamplerDescriptor` for more information.
        sampler_descriptor: wgpu::SamplerDescriptor<'static>,
        size: Extent3d,
        data: Vec<u8>,
    },
}

impl TextureDescriptor {
    pub fn uniform_rgba_white(srgb: bool) -> Self {
        Self::uniform_rgba_color(
            Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            srgb,
        )
    }
    pub fn uniform_rgba_black(srgb: bool) -> Self {
        Self::uniform_rgba_color(
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            srgb,
        )
    }
    pub fn uniform_rgba_gray(srgb: bool) -> Self {
        Self::uniform_rgba_color(
            Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 1.0,
            },
            srgb,
        )
    }

    pub fn uniform_rgba_color(color: Color, srgb: bool) -> Self {
        Self::Data {
            pixels: vec![
                ((color.r.clamp(0.0, 1.0)) * 255.0) as u8,
                ((color.g.clamp(0.0, 1.0)) * 255.0) as u8,
                ((color.b.clamp(0.0, 1.0)) * 255.0) as u8,
                255u8,
            ],
            size: TextureSize {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
                base_mip: 0,
                mip_levels: 1,
            },
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            format: if srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            },
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::default(),
        }
    }

    pub fn uniform_rgba_value(r: f64, g: f64, b: f64, a: f64, srgb: bool) -> Self {
        Self::uniform_rgba_color(Color { r, g, b, a }, srgb)
    }

    pub fn uniform_luma_black() -> Self {
        Self::Data {
            pixels: vec![0u8],
            size: TextureSize {
                width: 1,
                height: 1,
                ..Default::default()
            },
            format: TextureFormat::R8Unorm,
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::default(),
        }
    }

    pub fn uniform_luma_white() -> Self {
        Self::Data {
            pixels: vec![255u8],
            size: TextureSize {
                width: 1,
                height: 1,
                ..Default::default()
            },
            format: TextureFormat::R8Unorm,
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::default(),
        }
    }

    pub fn uniform_luma_gray() -> Self {
        Self::Data {
            pixels: vec![128u8],
            size: TextureSize {
                width: 1,
                height: 1,
                ..Default::default()
            },
            format: TextureFormat::R8Unorm,
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,

            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::default(),
        }
    }

    pub fn uniform_luma_value(value: f64) -> Self {
        Self::Data {
            pixels: vec![((value.clamp(0.0, 1.0)) * 255.0) as u8],
            size: TextureSize {
                width: 1,
                height: 1,
                ..Default::default()
            },
            format: TextureFormat::R8Unorm,
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::D2,
            filter_mode: FilterMode::default(),
        }
    }
}

impl Eq for TextureDescriptor {}

impl Hash for TextureDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TextureDescriptor::File { path, usages } => {
                path.hash(state);
                usages.hash(state);
            }
            TextureDescriptor::Data {
                pixels,
                size,
                format,
                usages,
                texture_dimension,
                texture_view_dimension,
                filter_mode,
            } => {
                pixels.hash(state);
                size.hash(state);
                format.hash(state);
                usages.hash(state);
                texture_dimension.hash(state);
                texture_view_dimension.hash(state);
                filter_mode.hash(state);
            }
            TextureDescriptor::Custom {
                texture_descriptor,
                view_descriptor,
                sampler_descriptor,
                size,
                data,
            } => {
                texture_descriptor.hash(state);

                view_descriptor.label.hash(state);
                view_descriptor.format.hash(state);
                view_descriptor.dimension.hash(state);
                view_descriptor.aspect.hash(state);
                view_descriptor.base_mip_level.hash(state);
                view_descriptor.mip_level_count.hash(state);
                view_descriptor.base_array_layer.hash(state);
                view_descriptor.array_layer_count.hash(state);

                sampler_descriptor.label.hash(state);
                sampler_descriptor.address_mode_u.hash(state);
                sampler_descriptor.address_mode_v.hash(state);
                sampler_descriptor.address_mode_w.hash(state);
                sampler_descriptor.mag_filter.hash(state);
                sampler_descriptor.min_filter.hash(state);
                sampler_descriptor.mipmap_filter.hash(state);
                sampler_descriptor.lod_min_clamp.to_le_bytes().hash(state);
                sampler_descriptor.lod_max_clamp.to_le_bytes().hash(state);
                sampler_descriptor.compare.hash(state);
                sampler_descriptor.anisotropy_clamp.hash(state);
                sampler_descriptor.border_color.hash(state);

                data.hash(state);
                size.hash(state);
            }
        }
    }
}
