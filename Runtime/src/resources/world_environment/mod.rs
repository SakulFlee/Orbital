use cgmath::Vector2;
use image::{GenericImageView, ImageReader};
use log::{debug, info, warn};
use std::error::Error;
use std::sync::OnceLock;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    BufferBindingType, BufferUsages, CommandEncoder, CompareFunction, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, Extent3d, FilterMode as WFilterMode,
    PipelineLayoutDescriptor, Queue, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderStages, StorageTextureAccess, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::mip_level::max_mip_level;
use crate::resources::{FilterMode, IblBrdf, MaterialShader, Texture, TextureSize};

mod error;
pub use error::*;

mod cache_file;
pub use cache_file::*;

mod skybox_type;
pub use skybox_type::*;

mod sampling_type;
pub use sampling_type::*;

mod descriptor;
pub use descriptor::*;

use super::{MaterialShaderDescriptor, ShaderSource, TextureDescriptor, VariableType};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct WorldEnvironment {
    /// IBL (= Image Based Lighting) diffuse Texture.
    /// To be used for illuminating objects in the current [`World`].
    ///
    /// _Should_ only contain a single LoD/MipMap.
    ibl_diffuse: Texture,
    /// IBL (= Image Based Lighting) specular Texture.
    /// To be used for sky box rendering and imitating reflections.
    ///
    /// _Should_ contain multiple LoD/MipMap's.
    /// Each LoD makes the sampled reflection blurrier and rougher (* if sampled correctly).
    ibl_specular: Texture,
    /// [`MaterialShader`] to be used with this [`WorldEnvironment`].
    material_shader: MaterialShader,
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

    pub fn find_cache_dir() -> PathBuf {
        dirs::cache_dir().expect("Could not find a valid cache location for the current platform! This platform might be unsupported ...")
    .join("Orbital").join("IBLs")
    }

    pub fn find_cache_file(descriptor: &WorldEnvironmentDescriptor) -> PathBuf {
        let cache_dir = Self::find_cache_dir();

        // Hash the descriptor to use as filename
        let mut hasher = DefaultHasher::new();
        descriptor.hash(&mut hasher);
        let hash = hasher.finish().to_string();

        return cache_dir.join(format!("{hash}.bin"));
    }

    pub fn from_descriptor(
        descriptor: &WorldEnvironmentDescriptor,
        surface_texture_format: Option<TextureFormat>,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Box<dyn Error>> {
        let cache_file = Self::find_cache_file(descriptor);

        // Try loading cache file
        let (pbr_ibl_diffuse, pbr_ibl_specular, write_to_cache) = match CacheFile::from_path(
            cache_file.clone(),
        ) {
            Ok(cache_file) => {
                let (pbr_ibl_diffuse, pbr_ibl_specular) =
                    cache_file.make_textures(descriptor, device, queue);

                info!("Using cached WorldEnvironment/IBL!");
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

                (pbr_ibl_diffuse, pbr_ibl_specular, false)
            }
            Err(e) => {
                warn!("WorldEnvironment::IBL cache failed to load, is corrupt or doesn't exist! Will continue generating IBL from HDRI. This may take a few seconds. Error: {e:?}");

                let (x, y) = Self::make_from_descriptor(descriptor, device, queue)?;
                (x, y, true)
            }
        };

        let shader = Self::make_material_shader(surface_texture_format, device, queue)?;

        let s = Self {
            ibl_diffuse: pbr_ibl_diffuse,
            ibl_specular: pbr_ibl_specular,
            material_shader: shader,
        };

        if write_to_cache {
            s.write_to_cache(&cache_file, device, queue)
                .expect("Failed to write to cache!");
        }

        Ok(s)
    }

    // TODO: Move into util
    fn calculate_specular_mip_level_count(
        cube_face_size: u32,
        requested_mip_level_count: Option<&u32>,
    ) -> u32 {
        let max_possible_mip_levels = cube_face_size.ilog2() + 1;
        let requested_mip_levels = requested_mip_level_count
            .copied()
            .unwrap_or(max_possible_mip_levels);
        let clamped_mip_levels = requested_mip_levels.min(max_possible_mip_levels);

        if let Some(requested) = requested_mip_level_count {
            if *requested > max_possible_mip_levels {
                warn!(
                    "Requested specular mip level count {requested} exceeds maximum possible {max_possible_mip_levels} for cube face size {cube_face_size}. Clamping to {clamped_mip_levels}."
                );
            }
        }

        clamped_mip_levels
    }

    pub fn make_from_descriptor(
        descriptor: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> Result<(Texture, Texture), Box<dyn Error>> {
        match descriptor {
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size,
                path,
                sampling_type,
                custom_specular_mip_level_count: specular_mip_level_count,
            } => {
                let clamped_mip_levels = Self::calculate_specular_mip_level_count(
                    *cube_face_size,
                    specular_mip_level_count.as_ref(),
                );

                Self::radiance_hdr_file(
                    path,
                    *cube_face_size,
                    sampling_type,
                    clamped_mip_levels,
                    device,
                    queue,
                )
            }
            WorldEnvironmentDescriptor::FromData {
                cube_face_size,
                data,
                size,
                sampling_type,
                specular_mip_level_count,
            } => {
                let clamped_mip_levels = Self::calculate_specular_mip_level_count(
                    *cube_face_size,
                    specular_mip_level_count.as_ref(),
                );

                Self::radiance_hdr_vec(
                    data,
                    *size,
                    *cube_face_size,
                    sampling_type,
                    clamped_mip_levels,
                    device,
                    queue,
                )
            }
        }
    }

    pub fn radiance_hdr_file(
        file_path: &str,
        dst_size: u32,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<(Texture, Texture), Box<dyn Error>> {
        let img = ImageReader::open(file_path)
            .map_err(WorldEnvironmentError::IO)?
            .decode()
            .map_err(WorldEnvironmentError::Image)?;

        let width = img.dimensions().0;
        let height = img.dimensions().1;

        let data = img
            .into_rgba32f()
            .iter()
            .map(|x| x.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();

        Self::radiance_hdr_vec(
            &data,
            Vector2 {
                x: width,
                y: height,
            },
            dst_size,
            sampling_type,
            specular_mip_level_count,
            device,
            queue,
        )
    }

    pub fn radiance_hdr_vec(
        data: &[u8],
        src_size: Vector2<u32>,
        dst_size: u32,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        device: &Device,
        queue: &Queue,
    ) -> Result<(Texture, Texture), Box<dyn Error>> {
        let src_texture = Texture::from_descriptors_and_data(
            &wgpu::TextureDescriptor {
                label: Some("Equirectangular SRC"),
                size: Extent3d {
                    width: src_size.x,
                    height: src_size.y,
                    ..Default::default()
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba32Float,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &TextureViewDescriptor::default(),
            &SamplerDescriptor {
                label: Some("Equirectangular SRC Sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: WFilterMode::Linear,
                min_filter: WFilterMode::Linear,
                mipmap_filter: WFilterMode::Linear,
                compare: Some(CompareFunction::Always),
                ..Default::default()
            },
            Some((
                data,
                Extent3d {
                    width: src_size.x,
                    height: src_size.y,
                    ..Default::default()
                },
            )),
            device,
            queue,
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
            specular_mip_level_count,
            &mut encoder,
            device,
        );
        let specular = Self::generate_specular_mip_maps(
            &raw_specular,
            sampling_type,
            specular_mip_level_count,
            &mut encoder,
            device,
        );

        queue.submit([encoder.finish()]);

        Ok((diffuse, specular))
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
            include_wgsl!("make_ibl_diffuse.wgsl"),
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
            1,
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
        let workgroups = dst_size.div_ceil(16);
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 6);

        dst_texture
    }

    fn make_ibl_specular(
        dst_size: u32,
        bind_group_layout: &BindGroupLayout,
        src_view: &TextureView,
        specular_mip_level_count: u32,
        encoder: &mut CommandEncoder,
        device: &Device,
    ) -> Texture {
        let pipeline = Self::make_compute_pipeline(
            &[bind_group_layout],
            include_wgsl!("make_ibl_specular.wgsl"),
            "main",
            device,
        );

        let max_mip_level = max_mip_level(dst_size);
        let specular_mip_level = if specular_mip_level_count > max_mip_level {
            warn!("Attempting to create specular texture with size {dst_size}, which gives a max allowed mip level of {max_mip_level}, but {specular_mip_level_count} was set! Defaulting to the maximum allowed value.");
            max_mip_level
        } else {
            specular_mip_level_count
        };

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
            specular_mip_level,
            device,
        );

        let dst_view = dst_texture.texture().create_view(&TextureViewDescriptor {
            label: Some("PBR IBL Specular --- !!! PROCESSING VIEW !!!"),
            dimension: Some(TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(1),
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
        let workgroups = dst_size.div_ceil(16);
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups, workgroups, 6);

        dst_texture
    }

    fn generate_specular_mip_maps(
        src_specular_ibl: &Texture,
        sampling_type: &SamplingType,
        specular_mip_level_count: u32,
        encoder: &mut CommandEncoder,
        device: &Device,
    ) -> Texture {
        let bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_mip_mapping());
        let mip_buffer_bind_group_layout =
            device.create_bind_group_layout(&Self::bind_group_layout_descriptor_buffer());

        let pipeline = Self::make_compute_pipeline(
            &[&bind_group_layout, &mip_buffer_bind_group_layout],
            include_wgsl!("make_mip_maps.wgsl"),
            "main",
            device,
        );

        let max_mip_levels = specular_mip_level_count;

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
            max_mip_levels,
            device,
        );

        for mip_level in 0..max_mip_levels {
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
            let workgroups = src_specular_ibl.texture().size().width.div_ceil(16);
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

    pub fn textures_to_texture_descriptors(
        pbr_ibl_diffuse: &Texture,
        pbr_ibl_specular: &Texture,
        device: &Device,
        queue: &Queue,
    ) -> (TextureDescriptor, TextureDescriptor) {
        let ibl_diffuse_data = pbr_ibl_diffuse.read_as_binary(device, queue);
        let ibl_diffuse_size = pbr_ibl_diffuse.texture().size();
        let ibl_diffuse_descriptor = TextureDescriptor::Data {
            pixels: ibl_diffuse_data,
            size: TextureSize {
                width: ibl_diffuse_size.width,
                height: ibl_diffuse_size.height,
                depth_or_array_layers: ibl_diffuse_size.depth_or_array_layers,
                base_mip: 0,
                mip_levels: pbr_ibl_diffuse.texture().mip_level_count(),
            },
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            format: pbr_ibl_diffuse.texture().format(),
            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::Cube,
            filter_mode: FilterMode::nearest(),
        };

        let ibl_specular_data = pbr_ibl_specular.read_as_binary(device, queue);
        let ibl_specular_size = pbr_ibl_specular.texture().size();
        let ibl_specular_descriptor = TextureDescriptor::Data {
            pixels: ibl_specular_data,
            size: TextureSize {
                width: ibl_specular_size.width,
                height: ibl_specular_size.height,
                depth_or_array_layers: ibl_specular_size.depth_or_array_layers,
                base_mip: 0,
                mip_levels: pbr_ibl_specular.texture().mip_level_count(),
            },
            usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            format: pbr_ibl_specular.texture().format(),

            texture_dimension: TextureDimension::D2,
            texture_view_dimension: TextureViewDimension::Cube,
            filter_mode: FilterMode::nearest(),
        };

        (ibl_diffuse_descriptor, ibl_specular_descriptor)
    }

    pub fn make_material_shader_descriptor() -> MaterialShaderDescriptor {
        MaterialShaderDescriptor {
            name: Some(String::from("WorldEnvironment MaterialShader")),
            shader_source: ShaderSource::String(include_str!("material_shader.wgsl")),
            variables: vec![],
            depth_stencil: false,
            vertex_stage_layouts: None,
            cull_mode: None,
            ..Default::default()
        }
    }

    fn make_material_shader(
        surface_texture_format: Option<TextureFormat>,
        device: &Device,
        queue: &Queue,
    ) -> Result<MaterialShader, Box<dyn Error>> {
        let descriptor = Self::make_material_shader_descriptor();

        MaterialShader::from_descriptor(&descriptor, surface_texture_format, device, queue)
    }

    pub fn write_to_cache(
        &self,
        cache_path: &PathBuf,
        device: &Device,
        queue: &Queue,
    ) -> Result<(), WorldEnvironmentError> {
        let ibl_diffuse_data = self.ibl_diffuse.read_as_binary(device, queue);
        let ibl_specular_data = self.ibl_specular.read_as_binary(device, queue);

        let cache_file = CacheFile {
            ibl_diffuse_data,
            ibl_specular_data,
        };
        cache_file.to_path(cache_path)
    }

    pub fn ibl_diffuse(&self) -> &Texture {
        &self.ibl_diffuse
    }

    pub fn ibl_specular(&self) -> &Texture {
        &self.ibl_specular
    }

    pub fn material_shader(&self) -> &MaterialShader {
        &self.material_shader
    }
}
