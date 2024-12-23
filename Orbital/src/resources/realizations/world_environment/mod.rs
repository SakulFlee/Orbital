use std::{
    collections::HashMap,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
};

use cgmath::Vector2;
use image::{GenericImageView, ImageReader};
use log::{debug, warn};
use serde::Serialize;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BufferBindingType, BufferUsages, CommandEncoder, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout,
    Origin3d, PipelineLayoutDescriptor, Queue, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderStages, StorageTextureAccess, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension,
};

use crate::{
    error::Error,
    resources::descriptors::{SamplingType, SkyboxType, WorldEnvironmentDescriptor},
};

use super::Texture;

mod cache_file;
pub use cache_file::*;

pub struct WorldEnvironment {
    skybox_type: SkyboxType,
    pbr_ibl_diffuse: Texture,
    pbr_ibl_specular: Texture,
}

impl WorldEnvironment {
    pub fn bind_group_layout_descriptor() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Equirectangular to PBR IBL Environment Maps"),
            entries: &[
                // Input: Equirectangular Image as source
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
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn bind_group_layout_descriptor_mip_mapping() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("PBR IBL Specular Environment Mip Mapping"),
            entries: &[
                // Input: PBR IBL Specular with LoD = 0 generated as source
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                // Src sampler
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Output
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn bind_group_layout_descriptor_buffer() -> BindGroupLayoutDescriptor<'static> {
        BindGroupLayoutDescriptor {
            label: Some("Mip Buffer Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        }
    }

    pub fn from_descriptor(
        descriptor: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
        app_name: &str,
    ) -> Result<Self, Error> {
        let mut hasher = DefaultHasher::new();
        descriptor.hash(&mut hasher);
        let hash = hasher.finish().to_string();

        if let Some(platform_cache_dir) = dirs::cache_dir() {
            let ibl_cache_file = platform_cache_dir
                .join(app_name)
                .join("IBLs")
                .join(format!("{}.bin", hash));
            debug!("WorldEnvironment/IBL Cache location: {:?}", ibl_cache_file);

            // Try loading cache file
            let cache_result = CacheFile::from_path(&ibl_cache_file);
            if let Ok(cache_file) = cache_result {
                let (pbr_ibl_diffuse, pbr_ibl_specular) =
                    cache_file.make_textures(descriptor, device, queue);

                debug!("Using cached WorldEnvironment/IBL!");
                debug!(
                    "Cached PBR IBL Diffuse Size: {:?} + Mip Levels: {:?}",
                    pbr_ibl_diffuse.texture().size(),
                    pbr_ibl_diffuse.texture().mip_level_count()
                );
                debug!(
                    "Cached PBR IBL Specular Size: {:?} + Mip Levels: {:?}",
                    pbr_ibl_specular.texture().size(),
                    pbr_ibl_specular.texture().mip_level_count()
                );

                return Ok(Self {
                    skybox_type: SkyboxType::Specular { lod: 0 },
                    pbr_ibl_diffuse,
                    pbr_ibl_specular,
                });
            }

            warn!(
                "WorldEnvironment/IBL cache failed to load! Generating new IBL ... Attempted cache status: {:?}",
                cache_result
            );

            // If cache doesn't exist, or failed to load, reset and regenerate
            let world_environment =
                Self::from_descriptor_without_disk_cache(descriptor, device, queue)?;

            let ibl_diffuse_data = world_environment
                .pbr_ibl_diffuse
                .read_as_binary(device, queue);
            let ibl_specular_data = world_environment
                .pbr_ibl_specular
                .read_as_binary(device, queue);

            let cache_file = CacheFile {
                ibl_diffuse_data,
                ibl_specular_data,
            };
            cache_file.to_path(ibl_cache_file)?;

            return Ok(world_environment);
        } else {
            warn!("Disk caching for WorldEnvironment/IBL is not supported on this platform! Loading the skybox might take significantly longer.");

            return Self::from_descriptor_without_disk_cache(descriptor, device, queue);
        }
    }

    pub fn from_descriptor_without_disk_cache(
        descriptor: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        match descriptor {
            WorldEnvironmentDescriptor::FromFile {
                skybox_type,
                cube_face_size,
                path,
                sampling_type,
            } => Self::radiance_hdr_file(
                *skybox_type,
                path,
                *cube_face_size,
                sampling_type,
                device,
                queue,
            ),
            WorldEnvironmentDescriptor::FromData {
                skybox_type,
                cube_face_size,
                data,
                size,
                sampling_type,
            } => Ok(Self::radiance_hdr_vec(
                *skybox_type,
                data,
                *size,
                *cube_face_size,
                sampling_type,
                device,
                queue,
            )),
        }
    }

    pub fn radiance_hdr_file(
        skybox_type: SkyboxType,
        file_path: &str,
        dst_size: u32,
        sampling_type: &SamplingType,
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
            skybox_type,
            &data,
            Vector2 {
                x: width,
                y: height,
            },
            dst_size,
            sampling_type,
            device,
            queue,
        ))
    }

    pub fn radiance_hdr_vec(
        skybox_type: SkyboxType,
        data: &[u8],
        src_size: Vector2<u32>,
        dst_size: u32,
        sampling_type: &SamplingType,
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

        let mut encoder = device.create_command_encoder(&Default::default());

        let diffuse = Self::make_ibl_diffuse(
            dst_size,
            &device.create_bind_group_layout(&Self::bind_group_layout_descriptor()),
            src_texture.view(),
            &mut encoder,
            device,
        );
        let raw_specular = Self::make_ibl_specular(
            dst_size,
            &device.create_bind_group_layout(&Self::bind_group_layout_descriptor()),
            src_texture.view(),
            &mut encoder,
            device,
        );
        let specular =
            Self::generate_specular_mip_maps(&raw_specular, sampling_type, &mut encoder, device);

        queue.submit([encoder.finish()]);

        Self {
            skybox_type,
            pbr_ibl_diffuse: diffuse,
            pbr_ibl_specular: specular,
        }
    }

    fn make_ibl_diffuse(
        dst_size: u32,
        bind_group_layout: &BindGroupLayout,
        src_view: &TextureView,
        encoder: &mut CommandEncoder,
        device: &Device,
    ) -> Texture {
        let pipeline = Self::make_compute_pipeline(
            &[bind_group_layout],
            include_wgsl!("world_environment_diffuse.wgsl"),
            "main",
            device,
        );

        let dst_texture = Texture::create_empty_cube_texture(
            Some("PBR IBL Diffuse"),
            Vector2 {
                x: dst_size,
                y: dst_size,
            },
            TextureFormat::Rgba16Float,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            false,
            device,
        );

        let dst_view = dst_texture.texture().create_view(&TextureViewDescriptor {
            label: Some("PBR IBL Diffuse --- !!! PROCESSING VIEW !!!"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Environment Processing Bind Group for PBR IBL Diffuse"),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(src_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Equirectangular Compute Task - Diffuse"),
            ..Default::default()
        });

        debug!("Generating PBR IBL Diffuse ...");
        let workgroups = (dst_size + 15) / 16;
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 6);

        dst_texture
    }

    fn make_ibl_specular(
        dst_size: u32,
        bind_group_layout: &BindGroupLayout,
        src_view: &TextureView,
        encoder: &mut CommandEncoder,
        device: &Device,
    ) -> Texture {
        let pipeline = Self::make_compute_pipeline(
            &[bind_group_layout],
            include_wgsl!("world_environment_specular.wgsl"),
            "main",
            device,
        );

        let dst_texture = Texture::create_empty_cube_texture(
            Some("PBR IBL Specular without LoDs"),
            Vector2 {
                x: dst_size,
                y: dst_size,
            },
            TextureFormat::Rgba16Float,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            false,
            device,
        );

        let dst_view = dst_texture.texture().create_view(&TextureViewDescriptor {
            label: Some("PBR IBL Specular --- !!! PROCESSING VIEW !!!"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Environment Processing Bind Group for PBR IBL Diffuse"),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(src_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Equirectangular Compute Task - Specular"),
            ..Default::default()
        });

        debug!("Generating RAW PBR IBL Specular (LoD = 0 / Roughness = 0%) ...");
        let workgroups = (dst_size + 15) / 16;
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 6);

        dst_texture
    }

    fn generate_specular_mip_maps(
        src_specular_ibl: &Texture,
        sampling_type: &SamplingType,
        encoder: &mut CommandEncoder,
        device: &Device,
    ) -> Texture {
        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_mip_mapping());
        let mip_buffer_bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_buffer());

        let pipeline = Self::make_compute_pipeline(
            &[&bind_group_layout, &mip_buffer_bind_group_layout],
            include_wgsl!("world_environment_mip_mapping.wgsl"),
            "main",
            device,
        );

        let dst_texture = Texture::create_empty_cube_texture(
            Some("PBR IBL Specular with LoDs"),
            Vector2 {
                x: src_specular_ibl.texture().width(),
                y: src_specular_ibl.texture().height(),
            },
            TextureFormat::Rgba16Float,
            TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            true,
            device,
        );

        let max_mip_levels = dst_texture.calculate_max_mip_levels() - 1;
        for mip_level in 0..=max_mip_levels {
            let dst_view = dst_texture.texture().create_view(&TextureViewDescriptor {
                label: Some("PBR IBL Specular LoD processing view"),
                dimension: Some(TextureViewDimension::D2Array),
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("World Environment Processing Bind Group for PBR IBL Diffuse"),
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(src_specular_ibl.view()),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(src_specular_ibl.sampler()),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(&dst_view),
                    },
                ],
            });

            let mip_bind_group = Self::make_mip_buffer(
                mip_level,
                max_mip_levels,
                sampling_type,
                &mip_buffer_bind_group_layout,
                device,
            );

            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("PBR IBL Specular Mip Mapping task"),
                ..Default::default()
            });

            debug!(
                "Generating PBR IBL Specular (LoD = {} / Roughness = {}%) ...",
                mip_level,
                (mip_level as f32 / max_mip_levels as f32) * 100.0
            );
            let workgroups = (src_specular_ibl.texture().size().width + 15) / 16;
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_bind_group(1, &mip_bind_group, &[]);
            pass.dispatch_workgroups(workgroups, workgroups, 6);
        }

        dst_texture
    }

    fn make_mip_buffer(
        mip_level: u32,
        max_mip_level: u32,
        sampling_type: &SamplingType,
        mip_buffer_bind_group_layout: &BindGroupLayout,
        device: &Device,
    ) -> BindGroup {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mip Buffer"),
            contents: &[
                mip_level.to_le_bytes(),
                max_mip_level.to_le_bytes(),
                sampling_type.to_le_bytes(),
            ]
            .concat(),
            usage: BufferUsages::UNIFORM,
        });

        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Mip Buffer Bind Group"),
            layout: mip_buffer_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            }],
        })
    }

    fn make_compute_pipeline(
        bind_group_layouts: &[&BindGroupLayout],
        shader_module_descriptor: ShaderModuleDescriptor,
        shader_entrypoint: &str,
        device: &Device,
    ) -> ComputePipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(shader_module_descriptor);

        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("WorldEnvironment Processing Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some(shader_entrypoint),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub fn pbr_ibl_diffuse(&self) -> &Texture {
        &self.pbr_ibl_diffuse
    }

    pub fn pbr_ibl_specular(&self) -> &Texture {
        &self.pbr_ibl_specular
    }

    pub fn skybox_type(&self) -> SkyboxType {
        self.skybox_type
    }
}
