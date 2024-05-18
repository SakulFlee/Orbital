use image::{DynamicImage, GenericImageView};
use log::warn;
use wgpu::Device;
use wgpu::Queue;
use wgpu::Texture as WTexture;
use wgpu::TextureDescriptor as WTextureDescriptor;
use wgpu::{
    AddressMode, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Sampler,
    SamplerDescriptor, TextureAspect, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use crate::resources::TextureDescriptor;

pub struct Texture {
    texture: WTexture,
    view: TextureView,
    sampler: Sampler,
}

impl Texture {
    pub fn from_descriptor(descriptor: &TextureDescriptor, device: &Device, queue: &Queue) -> Self {
        match descriptor {
            TextureDescriptor::StandardSRGBu8Image(image) => {
                Self::standard_srgb8_image(&image, device, queue)
            }
            TextureDescriptor::StandardSRGBu8Data(data, size) => {
                Self::standard_srgb8_data(data, size, device, queue)
            }
            TextureDescriptor::Custom(
                texture_descriptor,
                texture_view_descriptor,
                sampler_descriptor,
            ) => Self::from_descriptors(
                texture_descriptor,
                texture_view_descriptor,
                sampler_descriptor,
                device,
                queue,
            ),
        }
    }

    pub fn standard_srgb8_image(image: &DynamicImage, device: &Device, queue: &Queue) -> Self {
        Self::standard_srgb8_data(
            &image.to_rgba8(),
            &(image.dimensions().0, image.dimensions().1),
            device,
            queue,
        )
    }

    pub fn standard_srgb8_data(
        data: &[u8],
        size: &(u32, u32),
        device: &Device,
        queue: &Queue,
    ) -> Self {
        Self::from_data_srgb8(
            data,
            &WTextureDescriptor {
                label: Some("Standard SRGB u8 Data Texture"),
                size: Extent3d {
                    width: size.0,
                    height: size.1,
                    ..Default::default()
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &TextureViewDescriptor::default(),
            &SamplerDescriptor {
                label: Some("Standard SRGB u8 Data Texture Sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Nearest,
                ..Default::default()
            },
            device,
            queue,
        )
    }

    pub fn from_image_srgb8(
        image: DynamicImage,
        texture_desc: &WTextureDescriptor,
        view_desc: &TextureViewDescriptor,
        sampler_desc: &SamplerDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        if texture_desc.size.width != image.dimensions().0
            || texture_desc.size.height != image.dimensions().1
        {
            warn!("Image supplied has different dimensions from what the texture description expects! This may lead to undefined behaviour.");
        }

        Self::from_data_srgb8(
            &image.to_rgba8(),
            texture_desc,
            view_desc,
            sampler_desc,
            device,
            queue,
        )
    }

    pub fn from_data_srgb8(
        data: &[u8],
        texture_desc: &WTextureDescriptor,
        view_desc: &TextureViewDescriptor,
        sampler_desc: &SamplerDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let texture = Self::from_descriptors(texture_desc, view_desc, sampler_desc, device, queue);

        queue.write_texture(
            ImageCopyTexture {
                texture: texture.texture(),
                aspect: TextureAspect::All,
                origin: Origin3d::ZERO,
                mip_level: 0,
            },
            data,
            ImageDataLayout {
                offset: 0,
                // 4 bytes (RGBA), times the width
                bytes_per_row: Some(4 * texture_desc.size.width),
                // ... times height
                rows_per_image: Some(texture_desc.size.height),
            },
            texture_desc.size,
        );

        texture
    }

    pub fn from_descriptors(
        texture_desc: &WTextureDescriptor,
        view_desc: &TextureViewDescriptor,
        sampler_desc: &SamplerDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let texture = device.create_texture(&texture_desc);
        let view = texture.create_view(&view_desc);
        let sampler = device.create_sampler(&sampler_desc);

        Self::from_existing(texture, view, sampler)
    }

    pub fn from_existing(texture: WTexture, view: TextureView, sampler: Sampler) -> Self {
        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn texture(&self) -> &WTexture {
        &self.texture
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }
}
