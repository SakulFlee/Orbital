use std::sync::{Mutex, OnceLock};

use log::info;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Device, Queue, TextureFormat,
};

use crate::{
    cache::Cache,
    error::Error,
    resources::{
        descriptors::{
            CubeTextureDescriptor, MaterialDescriptor, PipelineDescriptor, ShaderDescriptor,
            TextureDescriptor,
        },
        realizations::CubeTexture,
    },
};

use super::{Pipeline, Texture};

#[derive(Debug)]
pub struct Material {
    bind_group: BindGroup,
    pipeline_descriptor: PipelineDescriptor,
    // TODO: Unsure if we can remove those after material creation?
    // normal_texture: Texture,
    // albedo_texture: Texture,
    // metallic_texture: Texture,
    // roughness_texture: Texture,
    // occlusion_texture: Texture,
}

impl Material {
    // --- Static ---
    /// Gives access to the internal pipeline cache.
    /// If the cache doesn't exist yet, it gets initialized.
    ///
    /// # Safety
    /// This is potentially a dangerous operation!
    /// The Rust compiler says the following:
    ///
    /// > use of mutable static is unsafe and requires unsafe function or block
    /// mutable statics can be mutated by multiple threads: aliasing violations
    /// or data races will cause undefined behavior
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
            MaterialDescriptor::SkyBox { sky_texture } => {
                Self::skybox(sky_texture, surface_format, device, queue)
            }
        })
    }

    pub fn skybox(
        sky_texture: &CubeTextureDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Error> {
        let cube_texture = CubeTexture::from_descriptor(sky_texture, device, queue)?;

        let pipeline_descriptor = PipelineDescriptor::default_skybox();
        let pipeline =
            Pipeline::from_descriptor(&pipeline_descriptor, surface_format, device, queue)?;

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: pipeline.bind_group_layout(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(cube_texture.texture().view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(cube_texture.texture().sampler()),
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

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: pipeline.bind_group_layout(),
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
                // OCCLUSION
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

        Ok(Self::from_existing(
            bind_group,
            pipeline_descriptor,
            // normal_texture,
            // albedo_texture,
            // metallic_texture,
            // roughness_texture,
            // occlusion_texture,
        ))
    }

    pub fn from_existing(
        bind_group: BindGroup,
        pipeline_descriptor: PipelineDescriptor,
        // normal_texture: Texture,
        // albedo_texture: Texture,
        // metallic_texture: Texture,
        // roughness_texture: Texture,
        // occlusion_texture: Texture,
    ) -> Self {
        Self {
            bind_group,
            pipeline_descriptor,
            // normal_texture,
            // albedo_texture,
            // metallic_texture,
            // roughness_texture,
            // occlusion_texture,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn pipeline_descriptor(&self) -> &PipelineDescriptor {
        &self.pipeline_descriptor
    }
}
