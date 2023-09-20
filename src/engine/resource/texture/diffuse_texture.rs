use std::path::Path;

use image::DynamicImage;
use wgpu::{
    AddressMode, Device, FilterMode, Queue, Sampler, SamplerDescriptor, Texture, TextureFormat,
    TextureView,
};

use crate::engine::{EngineResult, ResourceManager};

use super::{AbstractTexture, TTexture};

pub struct DiffuseTexture {
    internal_texture: AbstractTexture,
}

impl DiffuseTexture {
    pub const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    pub const SAMPLER_DESCRIPTOR: SamplerDescriptor<'static> = SamplerDescriptor {
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Nearest,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
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
                label,
            )?,
        })
    }
}

impl TTexture for DiffuseTexture {
    fn get_texture(&self) -> &Texture {
        &self.internal_texture.get_texture()
    }

    fn get_view(&self) -> &TextureView {
        &self.internal_texture.get_view()
    }

    fn get_sampler(&self) -> &Sampler {
        &self.internal_texture.get_sampler()
    }
}

impl ResourceManager {
    pub fn diffuse_texture_from_path<P>(
        device: &Device,
        queue: &Queue,
        file_name: P,
    ) -> EngineResult<DiffuseTexture>
    where
        P: AsRef<Path>,
    {
        DiffuseTexture::from_path(device, queue, file_name)
    }

    pub fn diffuse_texture_from_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &Vec<u8>,
        label: Option<&str>,
    ) -> EngineResult<DiffuseTexture> {
        DiffuseTexture::from_bytes(device, queue, bytes, label)
    }

    pub fn diffuse_texture_from_image(
        device: &Device,
        queue: &Queue,
        image: &DynamicImage,
        label: Option<&str>,
    ) -> EngineResult<DiffuseTexture> {
        DiffuseTexture::from_image(device, queue, image, label)
    }
}
