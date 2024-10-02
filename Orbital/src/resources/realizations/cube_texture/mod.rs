use cgmath::Vector2;
use image::{GenericImageView, ImageReader};
use wgpu::{
    include_wgsl, AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, PipelineLayoutDescriptor, Queue, SamplerDescriptor, ShaderStages, StorageTextureAccess, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension
};

use crate::{error::Error, resources::descriptors::CubeTextureDescriptor};

use super::Texture;

pub struct CubeTexture {
    texture: Texture,
}

impl CubeTexture {
    pub fn from_descriptor(
        desc: &CubeTextureDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        match desc {
            CubeTextureDescriptor::RadianceHDRFile {
                cube_face_size,
                path,
            } => Self::radiance_hdr_file(path, *cube_face_size, device, queue),
            CubeTextureDescriptor::RadianceHDRData {
                cube_face_size,
                data,
                size,
            } => Ok(Self::radiance_hdr_vec(
                data,
                *size,
                *cube_face_size,
                device,
                queue,
            )),
        }
    }

    pub fn bind_group_layout_descriptor() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Equirectangular"),
            entries: &[
                // Input
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Output
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        }
    }

    fn create_empty(
        label: Option<&str>,
        size: Vector2<u32>,
        format: TextureFormat,
        usage: TextureUsages,
        device: &Device,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width: size.x,
                height: size.y,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
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
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture: Texture::from_existing(texture, view, sampler),
        }
    }

    pub fn radiance_hdr_file(
        file_path: &str,
        dst_size: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let img = ImageReader::open(file_path)
            .map_err(Error::IOError)?
            .decode()
            .map_err(Error::ImageError)?;

        let width = img.dimensions().0;
        let height = img.dimensions().1;

        let data = img
            .into_rgba32f()
            .iter()
            .map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();

        Ok(Self::radiance_hdr_vec(
            &data,
            Vector2 {
                x: width,
                y: height,
            },
            dst_size,
            device,
            queue,
        ))
    }

    pub fn radiance_hdr_vec(
        data: &[u8],
        src_size: Vector2<u32>,
        dst_size: u32,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let src_texture = Texture::make_texture(
            Some("Equirectangular SRC"),
            Vector2 {
                x: src_size.x,
                y: src_size.y,
            },
            TextureFormat::Rgba32Float,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            FilterMode::Linear,
            AddressMode::ClampToEdge,
            device,
            queue,
        );

        queue.write_texture(
            ImageCopyTexture {
                texture: src_texture.texture(),
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(src_size.x * std::mem::size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(src_size.y),
            },
            Extent3d {
                width: src_size.x,
                height: src_size.y,
                ..Default::default()
            },
        );

        let dst_texture = Self::create_empty(
            Some("Equirectangular DST"),
            Vector2 {
                x: dst_size,
                y: dst_size,
            },
            TextureFormat::Rgba32Float,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            device,
        );

        // Needed only know for processing.
        // Actual rendering later uses the included View in
        // TextureViewDimension::Cube
        let dst_equirectangular_view =
            dst_texture
                .texture()
                .texture()
                .create_view(&TextureViewDescriptor {
                    label: Some("Equirectangular DST view"),
                    dimension: Some(TextureViewDimension::D2Array),
                    ..Default::default()
                });

        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Equirectangular"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(src_texture.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&dst_equirectangular_view),
                },
            ],
        });

        let pipeline = Self::make_pipeline(&bind_group_layout, device);

        let mut encoder = device.create_command_encoder(&Default::default());
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Equirectangular"),
            ..Default::default()
        });

        let workgroups = (dst_size + 15) / 16;
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 6);

        drop(pass);
        queue.submit([encoder.finish()]);

        dst_texture
    }

    fn make_pipeline(bind_group_layout: &BindGroupLayout, device: &Device) -> ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(include_wgsl!("equirectangular.wgsl"));

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Equirectangular to CubeMap"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}
