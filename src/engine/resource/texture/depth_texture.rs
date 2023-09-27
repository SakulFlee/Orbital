use std::path::Path;

use image::DynamicImage;
use wgpu::{
    AddressMode, CompareFunction, Device, Extent3d, FilterMode, Queue, Sampler, SamplerDescriptor,
    Texture, TextureFormat, TextureUsages, TextureView,
};

use crate::engine::EngineResult;

use super::{AbstractTexture, TTexture};

pub struct DepthTexture {
    internal_texture: AbstractTexture,
}

impl DepthTexture {
    pub const TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;
    pub const SAMPLER_DESCRIPTOR: SamplerDescriptor<'static> = SamplerDescriptor {
        label: Some("Depth Texture Sampler Descriptor"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        compare: Some(CompareFunction::LessEqual),
        lod_min_clamp: 0.0,
        lod_max_clamp: 100.0,
        anisotropy_clamp: 1, // Default
        border_color: None,  // Default
    };

    pub fn from_path<P>(device: &Device, queue: &Queue, file_name: P) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            internal_texture: AbstractTexture::from_path(
                device,
                queue,
                Self::TEXTURE_FORMAT,
                &Self::SAMPLER_DESCRIPTOR,
                TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
                file_name,
            )?,
        })
    }

    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &Vec<u8>,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        Ok(Self {
            internal_texture: AbstractTexture::from_bytes(
                device,
                queue,
                bytes,
                Self::TEXTURE_FORMAT,
                &Self::SAMPLER_DESCRIPTOR,
                TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
                label,
            )?,
        })
    }

    pub fn from_image(
        device: &Device,
        queue: &Queue,
        image: &DynamicImage,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        Ok(Self {
            internal_texture: AbstractTexture::from_image(
                device,
                queue,
                image,
                Self::TEXTURE_FORMAT,
                &Self::SAMPLER_DESCRIPTOR,
                TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
                label,
            )?,
        })
    }

    pub fn from_empty(
        device: &Device,
        size: Extent3d,
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        Ok(Self {
            internal_texture: AbstractTexture::from_empty(
                device,
                size,
                format,
                sampler_descriptor,
                TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
                label,
            )?,
        })
    }
}

impl TTexture for DepthTexture {
    fn get_texture(&self) -> &Texture {
        self.internal_texture.get_texture()
    }

    fn get_view(&self) -> &TextureView {
        self.internal_texture.get_view()
    }

    fn get_sampler(&self) -> &Sampler {
        self.internal_texture.get_sampler()
    }
}
