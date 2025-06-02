use std::ffi::OsString;

use cgmath::{Vector2, Vector4};
use image::ImageReader;
use wgpu::{
    AddressMode, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device, Extent3d,
    FilterMode, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, Sampler,
    SamplerDescriptor, Texture as WTexture, TextureAspect, TextureDescriptor as WTextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension,
};

mod size;
pub use size::*;

mod error;
pub use error::*;

mod descriptor;
pub use descriptor::*;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Texture {
    texture: WTexture,
    view: TextureView,
    sampler: Sampler,
    view_dimension: TextureViewDimension,
}

impl Texture {
    pub fn from_descriptor(
        descriptor: &TextureDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, TextureError> {
        match descriptor {
            TextureDescriptor::File { path, usages } => {
                Self::from_path(path, *usages, device, queue)
            }
            TextureDescriptor::Data {
                pixels,
                size,
                format,
                usages,
            } => Ok(Self::from_data(
                pixels, size, *usages, *format, device, queue,
            )),
            TextureDescriptor::Custom {
                texture_descriptor,
                view_descriptor,
                sampler_descriptor,
                size,
                data,
            } => Ok(Self::from_descriptors_and_data(
                texture_descriptor,
                view_descriptor,
                sampler_descriptor,
                Some((data, *size)),
                device,
                queue,
            )),
        }
    }

    pub fn create_empty_cube_texture(
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        mip_levels: u32,
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
            mip_level_count: mip_levels,
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

        Texture::from_existing(texture, view, sampler, TextureViewDimension::Cube)
    }

    pub fn from_binary_data(
        data: &[u8],
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        mip_levels: u32,
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
            mip_levels,
            device,
        );

        let mut data_offset = 0;
        for mip_level in 0..mip_levels {
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

    pub fn from_path(
        file_path: &OsString,
        usages: TextureUsages,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, TextureError> {
        let img = ImageReader::open(file_path)
            .map_err(TextureError::IOError)?
            .decode()
            .map_err(TextureError::ImageError)?;

        let data = img
            .to_rgba8()
            .iter()
            .map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();

        Self::from_descriptor(
            &TextureDescriptor::Data {
                pixels: data,
                size: TextureSize {
                    width: 1,
                    height: 1,
                    ..Default::default()
                },
                format: TextureFormat::Rgba8UnormSrgb,
                usages,
            },
            device,
            queue,
        )
    }

    /// In case you want a uniform, one color, image.
    /// This results in an 1-by-1 px, i.e. 4 bytes image.
    ///
    /// ⚠️ This can be used as an empty texture as there is as minimal
    /// ⚠️ as possible data usage and this resource may not even arrive
    /// ⚠️ in the shader _if_ it is not used.
    pub fn uniform_color(
        color: Vector4<u8>,
        usages: TextureUsages,
        format: TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        Self::from_data(
            &[color.x, color.y, color.z, color.w],
            &TextureSize {
                width: 1,
                height: 1,
                ..Default::default()
            },
            usages,
            format,
            device,
            queue,
        )
    }

    pub fn from_data(
        pixels: &[u8],
        size: &TextureSize,
        usages: TextureUsages,
        format: TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let texture_descriptor = wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: size.depth_or_array_layers,
            },
            format,
            mip_level_count: size.mip_levels,
            sample_count: 1,
            // TODO: Other dimensions cannot be set atm!
            dimension: TextureDimension::D2,
            usage: usages,
            view_formats: &[],
        };
        let texture = device.create_texture(&texture_descriptor);

        let texture_view_descriptor = wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            // TODO: CubeMap cannot be set!
            // TODO: 2D Array cannot be set!
            dimension: None,
            aspect: TextureAspect::All,
            base_mip_level: size.base_mip,
            mip_level_count: size.mip_levels.gt(&1).then_some(size.mip_levels),
            ..Default::default()
        };
        let texture_view = texture.create_view(&texture_view_descriptor);

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            // TODO: AddressMode cannot be changed!
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            // TODO: FilterMode cannot be changed!
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            // TODO: Min/Max clamping cannot be changed!
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        // Create actual orbital texture
        let view_dimension = texture_view_descriptor.dimension.unwrap_or({
            match texture_descriptor.dimension {
                TextureDimension::D1 => TextureViewDimension::D1,
                TextureDimension::D2 => TextureViewDimension::D2,
                TextureDimension::D3 => TextureViewDimension::D3,
            }
        });
        let texture = Self::from_existing(texture, texture_view, texture_sampler, view_dimension);

        // Write the data into the texture buffer
        queue.write_texture(
            ImageCopyTexture {
                texture: texture.texture(),
                aspect: TextureAspect::All,
                origin: Origin3d::ZERO,
                mip_level: 0,
            },
            pixels,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    size.width
                        * format.target_pixel_byte_cost().expect(
                            "Need to acquire target pixel byte cost for correct texture mapping!",
                        ),
                ),
                rows_per_image: Some(size.height),
            },
            Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: size.depth_or_array_layers,
            },
        );

        texture
    }

    pub fn depth_texture(size: &Vector2<u32>, device: &Device, queue: &Queue) -> Texture {
        Self::from_descriptors_and_data(
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
            None,
            device,
            queue,
        )
    }

    pub fn from_descriptors_and_data(
        texture_descriptor: &WTextureDescriptor,
        view_descriptor: &TextureViewDescriptor,
        sampler_descriptor: &SamplerDescriptor,
        data_and_size: Option<(&[u8], Extent3d)>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let texture_format = texture_descriptor.format;

        let texture = device.create_texture(texture_descriptor);
        let view = texture.create_view(view_descriptor);
        let sampler = device.create_sampler(sampler_descriptor);

        let view_dimension =
            view_descriptor
                .dimension
                .unwrap_or(match texture_descriptor.dimension {
                    TextureDimension::D1 => TextureViewDimension::D1,
                    TextureDimension::D2 => TextureViewDimension::D2,
                    TextureDimension::D3 => TextureViewDimension::D3,
                });
        let self_texture = Self::from_existing(texture, view, sampler, view_dimension);

        if let Some((data, size)) = data_and_size {
            queue.write_texture(
                ImageCopyTexture {
                    texture: self_texture.texture(),
                    aspect: TextureAspect::All,
                    origin: Origin3d::ZERO,
                    mip_level: 0,
                },
                data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        size.width * texture_format.target_pixel_byte_cost().expect("Need to acquire target pixel byte cost for correct texture mapping!",
                        ),
                    ),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        self_texture
    }

    pub fn from_existing(
        texture: WTexture,
        view: TextureView,
        sampler: Sampler,
        view_dimension: TextureViewDimension,
    ) -> Self {
        Self {
            texture,
            view,
            sampler,
            view_dimension,
        }
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

    pub fn view_dimension(&self) -> &TextureViewDimension {
        &self.view_dimension
    }
}
