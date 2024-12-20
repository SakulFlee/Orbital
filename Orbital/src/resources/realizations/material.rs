use std::sync::Arc;

use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, BufferBindingType, BufferUsages, Device, Queue, SamplerBindingType, ShaderStages,
    TextureFormat, TextureSampleType, TextureViewDimension,
};

use crate::{
    cache::{Cache, CacheEntry},
    error::Error,
    resources::{
        descriptors::{
            MaterialDescriptor, PipelineBindGroupLayout, PipelineDescriptor, ShaderDescriptor,
            SkyboxType, TextureDescriptor, WorldEnvironmentDescriptor,
        },
        realizations::WorldEnvironment,
    },
};

use super::{IblBrdf, Pipeline, Shader, Texture};

#[derive(Debug)]
pub struct Material {
    bind_group: BindGroup,
    pipeline_descriptor: Arc<PipelineDescriptor>,
    pipeline: Arc<Pipeline>,
}

impl Material {
    pub const PBR_PIPELINE_BIND_GROUP_NAME: &'static str = "PBR";
    pub const WORLD_ENVIRONMENT_PIPELINE_BIND_GROUP_NAME: &'static str = "WorldEnvironment";

    pub unsafe fn get_or_generate_ibl_brdf_lut(device: &Device, queue: &Queue) -> Texture {
        let ibl_brdf = IblBrdf::generate(device, queue);
        ibl_brdf.texture()
    }

    pub fn pbr_pipeline_bind_group_layout() -> PipelineBindGroupLayout {
        PipelineBindGroupLayout {
            label: Self::PBR_PIPELINE_BIND_GROUP_NAME,
            entries: vec![
                // Normal
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Albedo Texture
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Metallic Texture
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Roughness Texture
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Occlusion
                BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 9,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Emissive
                BindGroupLayoutEntry {
                    binding: 10,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 11,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Factors
                BindGroupLayoutEntry {
                    binding: 12,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn world_environment_pipeline_bind_group_layout() -> PipelineBindGroupLayout {
        PipelineBindGroupLayout {
            label: Self::WORLD_ENVIRONMENT_PIPELINE_BIND_GROUP_NAME,
            entries: vec![
                // Diffuse
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // Specular
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // IBL BRDF LUT
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Skybox info
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        }
    }

    pub fn from_descriptor(
        descriptor: &MaterialDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
        with_texture_cache: Option<&mut Cache<Arc<TextureDescriptor>, Texture>>,
        with_pipeline_cache: Option<&mut Cache<Arc<PipelineDescriptor>, Pipeline>>,
        with_shader_cache: Option<&mut Cache<Arc<ShaderDescriptor>, Shader>>,
    ) -> Result<Self, Error> {
        match descriptor {
            MaterialDescriptor::PBR {
                normal,
                albedo,
                albedo_factor,
                metallic,
                metallic_factor,
                roughness,
                roughness_factor,
                occlusion,
                emissive,
                custom_shader,
            } => Self::standard_pbr(
                normal.clone(),
                albedo.clone(),
                albedo_factor,
                metallic.clone(),
                metallic_factor,
                roughness.clone(),
                roughness_factor,
                occlusion.clone(),
                emissive.clone(),
                custom_shader.as_ref(),
                surface_format,
                device,
                queue,
                with_texture_cache,
                with_pipeline_cache,
                with_shader_cache,
            ),
            // Note that, WorldEnvironment doesn't use the Texture Cache as it works a lot different from normal Textures and Materials.
            // There also hardly is a need for a cache though, as only ever one `WorldEnvironment` is used at a time and while switching is possible, it shouldn't be switched so often that a cache will be needed.
            MaterialDescriptor::WorldEnvironment(world_environment) => Self::skybox(
                &world_environment,
                surface_format,
                device,
                queue,
                with_pipeline_cache,
                with_shader_cache,
            ),
        }
    }

    pub fn skybox(
        world_environment_descriptor: &WorldEnvironmentDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
        with_pipeline_cache: Option<&mut Cache<Arc<PipelineDescriptor>, Pipeline>>,
        with_shader_cache: Option<&mut Cache<Arc<ShaderDescriptor>, Shader>>,
    ) -> Result<Self, Error> {
        let world_environment =
            WorldEnvironment::from_descriptor(world_environment_descriptor, device, queue)?;
        let ibl_brdf_lut = unsafe { Self::get_or_generate_ibl_brdf_lut(device, queue) };

        let pipeline_descriptor = Arc::new(PipelineDescriptor::default_skybox());
        let pipeline = if let Some(cache) = with_pipeline_cache {
            cache
                .entry(pipeline_descriptor.clone())
                .or_insert(CacheEntry::new(Pipeline::from_descriptor(
                    &pipeline_descriptor,
                    surface_format,
                    device,
                    queue,
                    with_shader_cache,
                )?))
                .clone_inner()
        } else {
            Arc::new(Pipeline::from_descriptor(
                &pipeline_descriptor,
                surface_format,
                device,
                queue,
                with_shader_cache,
            )?)
        };

        let bind_group_layout = pipeline
            .bind_group_layout(Self::WORLD_ENVIRONMENT_PIPELINE_BIND_GROUP_NAME)
            .ok_or(Error::BindGroupMissing)?;

        let info_buffer_bytes = match world_environment.skybox_type() {
            SkyboxType::Diffuse => &[(-1i32).to_le_bytes()],
            SkyboxType::Specular { lod } => &[(lod as u32).to_le_bytes()],
        };

        let info_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Skybox Info Buffer"),
            contents: &info_buffer_bytes.concat(),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                // Diffuse
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        world_environment.pbr_ibl_diffuse().view(),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(
                        world_environment.pbr_ibl_diffuse().sampler(),
                    ),
                },
                // Specular
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(
                        world_environment.pbr_ibl_specular().view(),
                    ),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(
                        world_environment.pbr_ibl_specular().sampler(),
                    ),
                },
                // IBL BRDF LUT
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(ibl_brdf_lut.view()),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Sampler(ibl_brdf_lut.sampler()),
                },
                // Info
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Buffer(info_buffer.as_entire_buffer_binding()),
                },
            ],
        });

        Ok(Self::from_existing(
            bind_group,
            pipeline_descriptor,
            pipeline,
        ))
    }

    pub fn standard_pbr(
        normal_texture_descriptor: Arc<TextureDescriptor>,
        albedo_texture_descriptor: Arc<TextureDescriptor>,
        albedo_factor: &Vector3<f32>,
        metallic_texture_descriptor: Arc<TextureDescriptor>,
        metallic_factor: &f32,
        roughness_texture_descriptor: Arc<TextureDescriptor>,
        roughness_factor: &f32,
        occlusion_texture_descriptor: Arc<TextureDescriptor>,
        emissive_texture_descriptor: Arc<TextureDescriptor>,
        shader_descriptor: Option<&ShaderDescriptor>,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
        mut with_texture_cache: Option<&mut Cache<Arc<TextureDescriptor>, Texture>>,
        with_pipeline_cache: Option<&mut Cache<Arc<PipelineDescriptor>, Pipeline>>,
        with_shader_cache: Option<&mut Cache<Arc<ShaderDescriptor>, Shader>>,
    ) -> Result<Self, Error> {
        let normal_texture = match with_texture_cache {
            Some(ref mut cache) => cache
                .entry(normal_texture_descriptor.clone())
                .or_insert(CacheEntry::new(Texture::from_descriptor(
                    &normal_texture_descriptor,
                    device,
                    queue,
                )?))
                .clone_inner(),
            None => Arc::new(Texture::from_descriptor(
                &normal_texture_descriptor,
                device,
                queue,
            )?),
        };
        let albedo_texture = match with_texture_cache {
            Some(ref mut cache) => cache
                .entry(albedo_texture_descriptor.clone())
                .or_insert(CacheEntry::new(Texture::from_descriptor(
                    &albedo_texture_descriptor,
                    device,
                    queue,
                )?))
                .clone_inner(),
            None => Arc::new(Texture::from_descriptor(
                &albedo_texture_descriptor,
                device,
                queue,
            )?),
        };
        let metallic_texture = match with_texture_cache {
            Some(ref mut cache) => cache
                .entry(metallic_texture_descriptor.clone())
                .or_insert(CacheEntry::new(Texture::from_descriptor(
                    &metallic_texture_descriptor,
                    device,
                    queue,
                )?))
                .clone_inner(),
            None => Arc::new(Texture::from_descriptor(
                &metallic_texture_descriptor,
                device,
                queue,
            )?),
        };
        let roughness_texture = match with_texture_cache {
            Some(ref mut cache) => cache
                .entry(roughness_texture_descriptor.clone())
                .or_insert(CacheEntry::new(Texture::from_descriptor(
                    &roughness_texture_descriptor,
                    device,
                    queue,
                )?))
                .clone_inner(),
            None => Arc::new(Texture::from_descriptor(
                &roughness_texture_descriptor,
                device,
                queue,
            )?),
        };
        let occlusion_texture = match with_texture_cache {
            Some(ref mut cache) => cache
                .entry(occlusion_texture_descriptor.clone())
                .or_insert(CacheEntry::new(Texture::from_descriptor(
                    &occlusion_texture_descriptor,
                    device,
                    queue,
                )?))
                .clone_inner(),
            None => Arc::new(Texture::from_descriptor(
                &occlusion_texture_descriptor,
                device,
                queue,
            )?),
        };
        let emissive_texture = match with_texture_cache {
            Some(ref mut cache) => cache
                .entry(emissive_texture_descriptor.clone())
                .or_insert(CacheEntry::new(Texture::from_descriptor(
                    &emissive_texture_descriptor,
                    device,
                    queue,
                )?))
                .clone_inner(),
            None => Arc::new(Texture::from_descriptor(
                &emissive_texture_descriptor,
                device,
                queue,
            )?),
        };

        let pipeline_descriptor = Arc::new(if let Some(shader_descriptor) = shader_descriptor {
            PipelineDescriptor::default_with_shader(shader_descriptor)
        } else {
            PipelineDescriptor::default()
        });

        let pipeline = if let Some(cache) = with_pipeline_cache {
            cache
                .entry(pipeline_descriptor.clone())
                .or_insert(CacheEntry::new(Pipeline::from_descriptor(
                    &pipeline_descriptor,
                    surface_format,
                    device,
                    queue,
                    with_shader_cache,
                )?))
                .clone_inner()
        } else {
            Arc::new(Pipeline::from_descriptor(
                &pipeline_descriptor,
                surface_format,
                device,
                queue,
                with_shader_cache,
            )?)
        };

        let bind_group_layout = pipeline
            .bind_group_layout(Self::PBR_PIPELINE_BIND_GROUP_NAME)
            .ok_or(Error::BindGroupMissing)?;

        let factor_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("PBR Factor Buffer"),
            contents: [
                // Albedo Factor
                albedo_factor.x.to_le_bytes(), // R
                albedo_factor.y.to_le_bytes(), // G
                albedo_factor.z.to_le_bytes(), // B
                // Metallic Factor
                metallic_factor.to_le_bytes(), // LUMA
                // Roughness Factor
                roughness_factor.to_le_bytes(), // LUMA
                // Padding to reach 32
                [0; 4],
                [0; 4],
                [0; 4],
            ]
            .as_flattened(),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                // Normal
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(normal_texture.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(normal_texture.sampler()),
                },
                // Albedo
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(albedo_texture.view()),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(albedo_texture.sampler()),
                },
                // Metallic
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(metallic_texture.view()),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Sampler(metallic_texture.sampler()),
                },
                // Roughness
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::TextureView(roughness_texture.view()),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::Sampler(roughness_texture.sampler()),
                },
                // Occlusion
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::TextureView(occlusion_texture.view()),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::Sampler(occlusion_texture.sampler()),
                },
                // Emissive
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::TextureView(emissive_texture.view()),
                },
                BindGroupEntry {
                    binding: 11,
                    resource: BindingResource::Sampler(emissive_texture.sampler()),
                },
                // Factors
                BindGroupEntry {
                    binding: 12,
                    resource: BindingResource::Buffer(factor_buffer.as_entire_buffer_binding()),
                },
            ],
        });

        Ok(Self::from_existing(
            bind_group,
            pipeline_descriptor,
            pipeline,
        ))
    }
    pub fn from_existing(
        bind_group: BindGroup,
        pipeline_descriptor: Arc<PipelineDescriptor>,
        pipeline: Arc<Pipeline>,
    ) -> Self {
        Self {
            bind_group,
            pipeline_descriptor,
            pipeline,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn pipeline_descriptor(&self) -> &PipelineDescriptor {
        &self.pipeline_descriptor
    }

    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }
}
