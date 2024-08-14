use std::sync::{Mutex, OnceLock};

use log::info;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingResource,
    BindingType, Device, Queue, SamplerBindingType, ShaderStages, TextureFormat, TextureSampleType,
    TextureViewDimension,
};

use crate::{
    cache::Cache,
    error::Error,
    resources::{
        descriptors::{
            CubeTextureDescriptor, MaterialDescriptor, PipelineBindGroupLayout, PipelineDescriptor,
            ShaderDescriptor, TextureDescriptor,
        },
        realizations::CubeTexture,
    },
};

use super::{Pipeline, Texture};

#[derive(Debug)]
pub struct Material {
    bind_group: BindGroup,
    pipeline_descriptor: PipelineDescriptor,
}

impl Material {
    pub const PBR_PIPELINE_BIND_GROUP_NAME: &'static str = "PBR";
    pub const WORLD_ENVIRONMENT_PIPELINE_BIND_GROUP_NAME: &'static str = "WorldEnvironment";

    // --- Static ---
    /// Gives access to the internal pipeline cache.
    /// If the cache doesn't exist yet, it gets initialized.
    ///
    /// # Safety
    /// This is potentially a dangerous operation!
    /// The Rust compiler says the following:
    ///
    /// > use of mutable static is unsafe and requires unsafe function or block
    /// > mutable statics can be mutated by multiple threads: aliasing violations
    /// > or data races will cause undefined behavior
    ///
    /// However, once initialized, the cell [OnceLock] should never change and
    /// thus this should be safe.
    ///
    /// Additionally, we utilize a [Mutex] to ensure that access to the
    /// cache map and texture format is actually exclusive.
    pub unsafe fn cache() -> &'static mut Cache<MaterialDescriptor, Material> {
        static mut CACHE: OnceLock<Mutex<Cache<MaterialDescriptor, Material>>> = OnceLock::new();

        if CACHE.get().is_none() {
            info!("Material cache doesn't exist! Initializing ...");
            let _ = CACHE.get_or_init(|| Mutex::new(Cache::new()));
        }

        CACHE
            .get_mut()
            .unwrap()
            .get_mut()
            .expect("Cache access violation!")
    }

    /// Makes sure the cache is in the right state before accessing.
    /// Should be ideally called before each cache access.
    /// Once per context is enough though!
    ///
    /// This will set some cache parameters, if they don't exist yet
    /// (e.g. in case of a new cache), and make sure the pipelines
    /// still match the correct surface texture formats.
    /// If needed, this will also attempt recompiling all pipelines
    /// (and thus their shaders) to match a different format!
    ///
    /// > ⚠️ This is a copy of [Pipeline::prepare_cache_access](crate::resources::realizations::Pipeline::prepare_cache_access), without the [TextureFormat] stuff.
    /// > This function currently doesn't really do anything, but is kept as-is in case we need to add some functionality here later + calling the unsafe static function.
    pub fn prepare_cache_access() -> &'static mut Cache<MaterialDescriptor, Material> {
        unsafe { Self::cache() }
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
                // Albedo
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
                // Metallic
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
                // Roughness
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
            ],
        }
    }

    pub fn world_environment_pipeline_bind_group_layout() -> PipelineBindGroupLayout {
        PipelineBindGroupLayout {
            label: Self::WORLD_ENVIRONMENT_PIPELINE_BIND_GROUP_NAME,
            entries: vec![
                // Sky
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Irradiance
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                    count: None,
                },
                // Radiance
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::Cube,
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
                // IBL BRDF LUT
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
            ],
        }
    }

    pub fn from_descriptor(
        descriptor: &MaterialDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<&'static Self, Error> {
        let cache = Self::prepare_cache_access();

        cache.get_or_add_fallible(descriptor, |k| match k {
            MaterialDescriptor::PBR {
                normal,
                albedo,
                metallic,
                roughness,
                occlusion,
            } => Self::standard_pbr(
                normal,
                albedo,
                metallic,
                roughness,
                occlusion,
                None,
                surface_format,
                device,
                queue,
            ),
            MaterialDescriptor::PBRCustomShader {
                normal,
                albedo,
                metallic,
                roughness,
                occlusion,
                custom_shader,
            } => Self::standard_pbr(
                normal,
                albedo,
                metallic,
                roughness,
                occlusion,
                Some(custom_shader),
                surface_format,
                device,
                queue,
            ),
            MaterialDescriptor::WorldEnvironment {
                sky,
                irradiance,
                radiance,
            } => Self::skybox(sky, irradiance, radiance, surface_format, device, queue),
        })
    }

    pub fn skybox(
        sky: &CubeTextureDescriptor,
        irradiance: &CubeTextureDescriptor,
        radiance: &CubeTextureDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let sky_texture = CubeTexture::from_descriptor(sky, device, queue)?;
        let irradiance_texture = CubeTexture::from_descriptor(irradiance, device, queue)?;
        let radiance_texture = CubeTexture::from_descriptor(radiance, device, queue)?;
        let ibl_brdf_lut = Texture::from_descriptor(
            &TextureDescriptor::FilePath("Assets/IBL_BRDF_LUT/ibl_brdf_lut.png"),
            device,
            queue,
        )?;

        let pipeline_descriptor = PipelineDescriptor::default_skybox();
        let pipeline =
            Pipeline::from_descriptor(&pipeline_descriptor, surface_format, device, queue)?;

        let bind_group_layout = pipeline
            .bind_group_layout(Self::WORLD_ENVIRONMENT_PIPELINE_BIND_GROUP_NAME)
            .ok_or(Error::BindGroupMissing)?;

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                // Sky
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(sky_texture.texture().view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sky_texture.texture().sampler()),
                },
                // Irradiance
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(irradiance_texture.texture().view()),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(irradiance_texture.texture().sampler()),
                },
                // Radiance
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::TextureView(radiance_texture.texture().view()),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Sampler(radiance_texture.texture().sampler()),
                },
                // IBL BRDF LUT
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::TextureView(ibl_brdf_lut.view()),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::Sampler(ibl_brdf_lut.sampler()),
                },
            ],
        });

        Ok(Self::from_existing(bind_group, pipeline_descriptor))
    }

    pub fn standard_pbr(
        normal_texture_descriptor: &TextureDescriptor,
        albedo_texture_descriptor: &TextureDescriptor,
        metallic_texture_descriptor: &TextureDescriptor,
        roughness_texture_descriptor: &TextureDescriptor,
        occlusion_texture_descriptor: &TextureDescriptor,
        shader_descriptor: Option<&ShaderDescriptor>,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let normal_texture = Texture::from_descriptor(normal_texture_descriptor, device, queue)?;
        let albedo_texture = Texture::from_descriptor(albedo_texture_descriptor, device, queue)?;
        let metallic_texture =
            Texture::from_descriptor(metallic_texture_descriptor, device, queue)?;
        let roughness_texture =
            Texture::from_descriptor(roughness_texture_descriptor, device, queue)?;
        let occlusion_texture =
            Texture::from_descriptor(occlusion_texture_descriptor, device, queue)?;

        let pipeline_descriptor = if let Some(shader_descriptor) = shader_descriptor {
            PipelineDescriptor::default_with_shader(shader_descriptor)
        } else {
            PipelineDescriptor::default()
        };

        let pipeline =
            Pipeline::from_descriptor(&pipeline_descriptor, surface_format, device, queue)?;

        let bind_group_layout = pipeline
            .bind_group_layout(Self::PBR_PIPELINE_BIND_GROUP_NAME)
            .ok_or(Error::BindGroupMissing)?;

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
            ],
        });

        Ok(Self::from_existing(bind_group, pipeline_descriptor))
    }

    pub fn from_existing(bind_group: BindGroup, pipeline_descriptor: PipelineDescriptor) -> Self {
        Self {
            bind_group,
            pipeline_descriptor,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn pipeline_descriptor(&self) -> &PipelineDescriptor {
        &self.pipeline_descriptor
    }
}
