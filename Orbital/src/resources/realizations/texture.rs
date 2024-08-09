use cgmath::{Vector2, Vector4};
use image::{DynamicImage, GenericImageView, ImageReader};
use log::{debug, warn};
use wgpu::{
    AddressMode, CompareFunction, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout,
    Origin3d, Queue, Sampler, SamplerDescriptor, Texture as WTexture, TextureAspect,
    TextureDescriptor as WTextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

use crate::{error::Error, resources::descriptors::TextureDescriptor};

pub struct Texture {
    texture: WTexture,
    view: TextureView,
    sampler: Sampler,
}

impl Texture {
    pub fn from_descriptor(
        descriptor: &TextureDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        match descriptor {
            TextureDescriptor::FilePath(file_path) => {
                Self::from_file_path(file_path, device, queue)
            }
            TextureDescriptor::StandardSRGBAu8Data(data, size) => {
                Ok(Self::standard_srgba8_data(data, size, device, queue))
            }
            TextureDescriptor::UniformColor(color) => {
                Ok(Self::uniform_color(*color, device, queue))
            }
            TextureDescriptor::Luma { data, size } => Ok(Self::luma(data, size, device, queue)),
            TextureDescriptor::UniformLuma { data } => Ok(Self::uniform_luma(data, device, queue)),
            TextureDescriptor::Depth(size) => Ok(Self::depth_texture(size, device, queue)),
        }
    }

    pub fn from_file_path(file_path: &str, device: &Device, queue: &Queue) -> Result<Self, Error> {
        let img = ImageReader::open(file_path)
            .map_err(Error::IOError)?
            .decode()
            .map_err(Error::ImageError)?;

        Ok(Self::standard_srgba8_data(
            img.as_bytes(),
            &(img.width(), img.height()).into(),
            device,
            queue,
        ))
    }

    /// In case you want a uniform, one color, image.
    /// This results in an 1-by-1 px, i.e. 4 bytes image.
    ///
    /// ⚠️ This can be used as an empty texture as there is as minimal
    /// ⚠️ as possible data usage and this resource may not even arrive
    /// ⚠️ in the shader _if_ it is not used.
    pub fn uniform_color(color: Vector4<u8>, device: &Device, queue: &Queue) -> Self {
        Self::standard_srgba8_data(
            &[color.x, color.y, color.z, color.w],
            &(1, 1).into(),
            device,
            queue,
        )
    }

    /// Luma (Grayscale) textures
    pub fn luma(data: &Vec<u8>, size: &Vector2<u32>, device: &Device, queue: &Queue) -> Self {
        let texture = Self::from_descriptors(
            &WTextureDescriptor {
                label: Some("Luma Texture"),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &TextureViewDescriptor::default(),
            &SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            },
            device,
            queue,
        );

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
                // 1 bytes (Luma), times the width
                bytes_per_row: Some(size.x),
                // ... times height
                rows_per_image: Some(size.y),
            },
            Extent3d {
                width: size.x,
                height: size.y,
                ..Default::default()
            },
        );

        texture
    }

    /// Uniform Luma (Grayscale) textures
    pub fn uniform_luma(data: &u8, device: &Device, queue: &Queue) -> Self {
        let texture = Self::from_descriptors(
            &WTextureDescriptor {
                label: Some("Luma Texture"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &TextureViewDescriptor::default(),
            &SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            },
            device,
            queue,
        );

        queue.write_texture(
            ImageCopyTexture {
                texture: texture.texture(),
                aspect: TextureAspect::All,
                origin: Origin3d::ZERO,
                mip_level: 0,
            },
            &[*data],
            ImageDataLayout {
                offset: 0,
                // 1 bytes (Luma), times the width
                bytes_per_row: Some(1),
                // ... times height
                rows_per_image: Some(1),
            },
            Extent3d {
                width: 1,
                height: 1,
                ..Default::default()
            },
        );

        texture
    }

    pub fn standard_srgba8_data(
        data: &[u8],
        size: &Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        Self::from_data_srgb8(
            data,
            &WTextureDescriptor {
                label: Some("Standard SRGB u8 Data Texture"),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
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
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
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

    pub fn depth_texture(size: &Vector2<u32>, device: &Device, queue: &Queue) -> Texture {
        Self::from_descriptors(
            &WTextureDescriptor {
                label: Some("Depth Texture"),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            &TextureViewDescriptor::default(),
            &SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            },
            device,
            queue,
        )
    }

    pub fn make_texture(
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        filter: FilterMode,
        address_mode: AddressMode,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        Self::from_descriptors(
            &WTextureDescriptor {
                label: label,
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    ..Default::default()
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            },
            &TextureViewDescriptor::default(),
            &SamplerDescriptor {
                label: Some("Radiance HDR Sampler"),
                address_mode_u: address_mode,
                address_mode_v: address_mode,
                address_mode_w: address_mode,
                mag_filter: filter,
                min_filter: filter,
                mipmap_filter: filter,
                compare: Some(CompareFunction::Always),
                ..Default::default()
            },
            device,
            queue,
        )
    }

    pub fn from_descriptors(
        texture_desc: &WTextureDescriptor,
        view_desc: &TextureViewDescriptor,
        sampler_desc: &SamplerDescriptor,
        device: &Device,
        _queue: &Queue,
    ) -> Self {
        let texture = device.create_texture(texture_desc);
        let view = texture.create_view(view_desc);
        let sampler = device.create_sampler(sampler_desc);

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
