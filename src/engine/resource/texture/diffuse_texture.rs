use std::path::Path;

use image::DynamicImage;
use wgpu::{
    AddressMode, Extent3d, FilterMode, Sampler, SamplerDescriptor, Texture, TextureFormat,
    TextureUsages, TextureView,
};

use crate::engine::{EngineResult, LogicalDevice, ResourceManager};

use super::{AbstractTexture, TTexture};

pub struct DiffuseTexture {
    internal_texture: AbstractTexture,
}

impl DiffuseTexture {
    pub const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    pub const SAMPLER_DESCRIPTOR: SamplerDescriptor<'static> = SamplerDescriptor {
        label: Some("Diffuse Texture Sampler Descriptor"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        lod_min_clamp: 0.0,  // Default
        lod_max_clamp: 32.0, // Default
        compare: None,       // Default
        anisotropy_clamp: 1, // Default
        border_color: None,  // Default
    };

    pub fn empty(
        logical_device: &LogicalDevice,
        special_format: Option<TextureFormat>,
    ) -> EngineResult<Self> {
        Ok(Self {
            internal_texture: AbstractTexture::from_empty(
                logical_device,
                Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                special_format.unwrap_or(Self::TEXTURE_FORMAT),
                &Self::SAMPLER_DESCRIPTOR,
                TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                Some("Empty"),
            )?,
        })
    }

    pub fn from_path<P>(
        logical_device: &LogicalDevice,
        file_path: P,
        special_format: Option<TextureFormat>,
    ) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            internal_texture: AbstractTexture::from_path(
                logical_device,
                special_format.unwrap_or(Self::TEXTURE_FORMAT),
                &Self::SAMPLER_DESCRIPTOR,
                TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                file_path,
            )?,
        })
    }

    pub fn from_bytes(
        logical_device: &LogicalDevice,
        bytes: &[u8],
        special_format: Option<TextureFormat>,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        Ok(Self {
            internal_texture: AbstractTexture::from_bytes(
                logical_device,
                bytes,
                special_format.unwrap_or(Self::TEXTURE_FORMAT),
                &Self::SAMPLER_DESCRIPTOR,
                TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                label,
            )?,
        })
    }

    pub fn from_image(
        logical_device: &LogicalDevice,
        image: &DynamicImage,
        special_format: Option<TextureFormat>,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        Ok(Self {
            internal_texture: AbstractTexture::from_image(
                logical_device,
                image,
                special_format.unwrap_or(Self::TEXTURE_FORMAT),
                &Self::SAMPLER_DESCRIPTOR,
                TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                label,
            )?,
        })
    }
}

impl TTexture for DiffuseTexture {
    fn texture(&self) -> &Texture {
        self.internal_texture.texture()
    }

    fn view(&self) -> &TextureView {
        self.internal_texture.view()
    }

    fn sampler(&self) -> &Sampler {
        self.internal_texture.sampler()
    }
}

impl ResourceManager {
    pub fn diffuse_texture_from_path<P>(
        logical_device: &LogicalDevice,
        file_path: P,
    ) -> EngineResult<DiffuseTexture>
    where
        P: AsRef<Path>,
    {
        DiffuseTexture::from_path(logical_device, file_path, None)
    }

    pub fn diffuse_texture_from_bytes(
        logical_device: &LogicalDevice,
        bytes: &[u8],
        label: Option<&str>,
    ) -> EngineResult<DiffuseTexture> {
        DiffuseTexture::from_bytes(logical_device, bytes, None, label)
    }

    pub fn diffuse_texture_from_image(
        logical_device: &LogicalDevice,
        image: &DynamicImage,
        label: Option<&str>,
    ) -> EngineResult<DiffuseTexture> {
        DiffuseTexture::from_image(logical_device, image, None, label)
    }
}
