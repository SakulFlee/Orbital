
use cgmath::{Vector2, Vector4};
use image::ImageReader;
use wgpu::{
    AddressMode, BufferDescriptor, BufferUsages, CommandEncoderDescriptor,
    CompareFunction, Device, Extent3d, FilterMode, ImageCopyBuffer, ImageCopyTexture,
    ImageDataLayout, Origin3d, Queue, Sampler, SamplerDescriptor, Texture as WTexture,
    TextureAspect, TextureDescriptor as WTextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::{error::Error, resources::descriptors::TextureDescriptor};

#[derive(Debug)]
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

    pub fn create_empty_cube_texture(
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        with_mips: bool,
        device: &Device,
    ) -> Texture {
        let size = Extent3d {
            width: size.x,
            height: size.y,
            // A cube has 6 sides, so we need 6 layers
            depth_or_array_layers: 6,
        };
        let texture = device.create_texture(&WTextureDescriptor {
            label,
            size,
            mip_level_count: if with_mips {
                Texture::calculate_max_mip_levels_from_texture_size(&size)
            } else {
                1
            },
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor {
            label,
            dimension: Some(TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        Texture::from_existing(texture, view, sampler)
    }

    pub fn from_binary_data(
        data: &[u8],
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        with_mips: bool,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        const ALIGN: u64 = 256;
        const BYTES_PER_PIXEL: u64 = 8; // Rgba16Float format

        let texture = Self::create_empty_cube_texture(
            label,
            size,
            format,
            usage | TextureUsages::COPY_DST,
            with_mips,
            device,
        );

        let mut data_offset = 0;
        let mip_count = if with_mips {
            texture.calculate_max_mip_levels()
        } else {
            1
        };

        for mip_level in 0..mip_count {
            let mip_size = Extent3d {
                width: size.x >> mip_level,
                height: size.y >> mip_level,
                depth_or_array_layers: 6,
            };

            let bytes_per_row = BYTES_PER_PIXEL * (mip_size.width as u64);
            let aligned_bytes_per_row = bytes_per_row.div_ceil(ALIGN) * ALIGN;

            let rows_per_layer = mip_size.height as u64;
            let bytes_per_layer = aligned_bytes_per_row * rows_per_layer;

            for layer_start in (0..mip_size.depth_or_array_layers).step_by(1) {
                let chunk_size = bytes_per_layer as usize;

                queue.write_texture(
                    ImageCopyTexture {
                        texture: texture.texture(),
                        mip_level,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: layer_start,
                        },
                        aspect: TextureAspect::All,
                    },
                    &data[data_offset..data_offset + chunk_size],
                    ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(aligned_bytes_per_row as u32),
                        rows_per_image: Some(mip_size.height),
                    },
                    Extent3d {
                        width: mip_size.width,
                        height: mip_size.height,
                        depth_or_array_layers: 1,
                    },
                );

                data_offset += chunk_size;
            }
        }

        texture
    }

    pub fn from_file_path(file_path: &str, device: &Device, queue: &Queue) -> Result<Self, Error> {
        let img = ImageReader::open(file_path)
            .map_err(Error::IOError)?
            .decode()
            .map_err(Error::ImageError)?;

        let data = img
            .to_rgba8()
            .iter()
            .map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();

        Ok(Self::standard_srgba8_data(
            &data,
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
                label,
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

    pub fn calculate_max_mip_levels(&self) -> u32 {
        Self::calculate_max_mip_levels_from_texture_size(&self.texture.size())
    }

    pub fn calculate_max_mip_levels_from_texture_size(texture_size: &Extent3d) -> u32 {
        let max_size = texture_size.width.max(texture_size.height) as f32;

        let size_log = max_size.log2();
        let size_floor = size_log.floor();

        (size_floor as u32) + 1
    }

    pub fn read_as_binary(&self, device: &Device, queue: &Queue) -> Vec<u8> {
        // 256MB, hard enforced by WGPU. A buffer cannot be bigger than this, thus we need to read in chunks.
        const MAX_BUFFER_SIZE: u64 = 268_435_456;
        // In WGPU a buffer has to be aligned with 2^8 bytes.
        const ALIGN: u64 = 256;
        // We use the Rgba16Float format, that's 16 bits per channel, thus 4 * 16 = 64 bits per whole pixel, which is 64 bits / 8 bits per byte = 8 bytes
        const BYTES_PER_PIXEL: u64 = 8;

        let mut final_data = Vec::new();
        let mip_count = self.texture.mip_level_count();

        for mip_level in 0..mip_count {
            let mip_size = Extent3d {
                width: self.texture.width() >> mip_level,
                height: self.texture.height() >> mip_level,
                depth_or_array_layers: self.texture.depth_or_array_layers(),
            };

            let bytes_per_row = BYTES_PER_PIXEL * (mip_size.width as u64);
            let aligned_bytes_per_row = bytes_per_row.div_ceil(ALIGN) * ALIGN;

            // Calculate how many complete layers we can fit in one buffer
            let rows_per_layer = mip_size.height as u64;
            let bytes_per_layer = aligned_bytes_per_row * rows_per_layer;
            let layers_per_chunk = (MAX_BUFFER_SIZE / bytes_per_layer).max(1);

            // Process layers in chunks
            for layer_start in
                (0..mip_size.depth_or_array_layers).step_by(layers_per_chunk as usize)
            {
                let layer_count = ((mip_size.depth_or_array_layers - layer_start) as u64)
                    .min(layers_per_chunk) as u32;
                let chunk_size = bytes_per_layer * layer_count as u64;

                let buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("Texture Read Buffer"),
                    size: chunk_size,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                let mut encoder =
                    device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                encoder.copy_texture_to_buffer(
                    ImageCopyTexture {
                        texture: &self.texture,
                        mip_level,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: layer_start,
                        },
                        aspect: TextureAspect::All,
                    },
                    ImageCopyBuffer {
                        buffer: &buffer,
                        layout: ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(aligned_bytes_per_row as u32),
                            rows_per_image: Some(mip_size.height),
                        },
                    },
                    Extent3d {
                        width: mip_size.width,
                        height: mip_size.height,
                        depth_or_array_layers: layer_count,
                    },
                );

                // Submit the "copy texture to buffer" command and wait for it to finish
                queue.submit([encoder.finish()]);
                device.poll(wgpu::Maintain::Wait);

                // Mark buffer as readable by mapping it and wait for it to finish
                buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
                device.poll(wgpu::Maintain::Wait);

                // Append our now readable data
                final_data.extend_from_slice(&buffer.slice(..).get_mapped_range());
            }
        }

        final_data
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
