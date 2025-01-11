use std::{
    hash::{Hash, Hasher},
    mem,
    sync::Arc,
};

use cgmath::Vector3;
use wgpu::Color;

use super::{SamplingType, ShaderDescriptor, TextureDescriptor, WorldEnvironmentDescriptor};

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialDescriptor {
    /// Creates a standard PBR (= Physically-Based-Rendering) material.
    PBR {
        normal: Arc<TextureDescriptor>,
        albedo: Arc<TextureDescriptor>,
        albedo_factor: Vector3<f32>,
        metallic: Arc<TextureDescriptor>,
        metallic_factor: f32,
        roughness: Arc<TextureDescriptor>,
        roughness_factor: f32,
        occlusion: Arc<TextureDescriptor>,
        emissive: Arc<TextureDescriptor>,
        custom_shader: Option<ShaderDescriptor>,
    },
    // TODO
    WorldEnvironment(WorldEnvironmentDescriptor),
    Wireframe(Color),
}

impl MaterialDescriptor {
    pub fn default_world_environment() -> MaterialDescriptor {
        MaterialDescriptor::WorldEnvironment(WorldEnvironmentDescriptor::FromFile {
            skybox_type: super::SkyboxType::Specular { lod: 0 },
            cube_face_size: super::WorldEnvironmentDescriptor::DEFAULT_SIZE,
            path: "Assets/HDRs/kloppenheim_02_puresky_4k.hdr",
            sampling_type: SamplingType::ImportanceSampling,
        })
    }
}

impl Hash for MaterialDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
}

impl Eq for MaterialDescriptor {}
