use std::{
    io::{BufReader, Cursor},
    path::Path,
};

use image::{io::Reader, DynamicImage, GenericImageView, ImageFormat};
use wgpu::{
    Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Sampler, SamplerDescriptor, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
};

use crate::engine::{EngineError, EngineResult, LogicalDevice, ResourceManager, TextureHelper};

use super::TTexture;

pub struct AbstractTexture {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
}

impl AbstractTexture {
    pub fn from_path<P>(
        logical_device: &LogicalDevice,
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        usage: TextureUsages,
        file_path: P,
    ) -> EngineResult<Self>
    where
        P: AsRef<Path>,
    {
        let file_name = file_path.as_ref().clone().to_str();
        let bytes = ResourceManager::read_resource_binary(file_path.as_ref().clone())?;

        Self::from_bytes(
            logical_device,
            &bytes,
            format,
            sampler_descriptor,
            usage,
            file_name,
        )
    }

    pub fn from_bytes(
        logical_device: &LogicalDevice,
        bytes: &[u8],
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        usage: TextureUsages,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        // let mut reader = Reader::new(Cursor::new(bytes));

        // reader.set_format(ImageFormat::Jpeg);

        // let image = reader.decode().map_err(|e| EngineError::ImageError(e))?;

        let image = image::load_from_memory(bytes).map_err(|e| EngineError::ImageError(e))?;

        Self::from_image(
            logical_device,
            &image,
            format,
            sampler_descriptor,
            usage,
            label,
        )
    }

    pub fn from_image(
        logical_device: &LogicalDevice,
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

        let abstract_texture = Self::from_empty(
            logical_device,
            size,
            format,
            sampler_descriptor,
            usage,
            label,
        )?;

        // Convert the image into something useable
        let rgba = image.to_rgba8();

        // Fill texture
        logical_device.queue().write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture: abstract_texture.texture(),
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
        logical_device: &LogicalDevice,
        size: Extent3d,
        format: TextureFormat,
        sampler_descriptor: &SamplerDescriptor,
        usage: TextureUsages,
        label: Option<&str>,
    ) -> EngineResult<Self> {
        // Make texture
        let texture = logical_device.device().create_texture(&TextureDescriptor {
            label,
            size,
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
        let sampler = logical_device.device().create_sampler(sampler_descriptor);

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}

impl TTexture for AbstractTexture {
    fn texture(&self) -> &Texture {
        &self.texture
    }

    fn view(&self) -> &TextureView {
        &self.view
    }

    fn sampler(&self) -> &Sampler {
        &self.sampler
    }
}
