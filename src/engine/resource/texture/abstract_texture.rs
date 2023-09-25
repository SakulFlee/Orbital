use std::path::Path;

use image::{DynamicImage, GenericImageView};
use wgpu::{
    Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Sampler,
    SamplerDescriptor, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView,
};

use crate::engine::{EngineError, EngineResult, ResourceManager, TextureHelper};

use super::TTexture;

pub struct AbstractTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

impl AbstractTexture {
    pub fn from_path<P>(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        usage: TextureUsages,
        file_path: P,
    ) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        let file_name = file_path
            .as_ref()
            .clone()
            .to_str()
            .map_or(None, |x| Some(x));
        let bytes = ResourceManager::read_resource_binary(file_path.as_ref().clone())?;

        Ok(Self::from_bytes(
            device,
            queue,
            &bytes,
            format,
            sampler_descriptor,
            usage,
            file_name,
        )?)
    }

    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &Vec<u8>,
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        usage: TextureUsages,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        let image = image::load_from_memory(bytes).map_err(|e| EngineError::ImageError(e))?;

        Self::from_image(
            device,
            queue,
            &image,
            format,
            sampler_descriptor,
            usage,
            label,
        )
    }

    pub fn from_image(
        device: &Device,
        queue: &Queue,
        image: &DynamicImage,
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        usage: TextureUsages,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        let dimensions = image.dimensions();
        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let abstract_texture =
            Self::from_empty(device, size, format, sampler_descriptor, usage, label)?;

        // Convert the image into something useable
        let rgba = image.to_rgba8();

        // Fill texture
        queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture: &abstract_texture.get_texture(),
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        Ok(abstract_texture)
    }

    pub fn from_empty(
        device: &Device,
        size: Extent3d,
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        usage: TextureUsages,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        // Make texture
        let texture = device.create_texture(&TextureDescriptor {
            label: label,
            size: size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        // Create texture view
        let view = texture.make_texture_view();

        // Create texture sampler
        let sampler = device.create_sampler(sampler_descriptor);

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}

impl TTexture for AbstractTexture {
    fn get_texture(&self) -> &Texture {
        &self.texture
    }

    fn get_view(&self) -> &TextureView {
        &self.view
    }

    fn get_sampler(&self) -> &Sampler {
        &self.sampler
    }
}
