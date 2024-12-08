use std::{
    hash::{Hash, Hasher},
    mem,
};

use cgmath::Vector3;

use super::{SamplingType, ShaderDescriptor, TextureDescriptor, WorldEnvironmentDescriptor};

#[derive(Debug, Clone, PartialEq)]
pub enum MaterialDescriptor {
    /// Creates a standard PBR (= Physically-Based-Rendering) material.
    PBR {
        normal: TextureDescriptor,
        albedo: TextureDescriptor,
        albedo_factor: Vector3<f32>,
        metallic: TextureDescriptor,
        metallic_factor: f32,
        roughness: TextureDescriptor,
        roughness_factor: f32,
        occlusion: TextureDescriptor,
        emissive: TextureDescriptor,
    },
    /// Creates a PBR (= Physically-Based-Rendering) material
    /// with a custom shader.
    PBRCustomShader {
        normal: TextureDescriptor,
        albedo: TextureDescriptor,
        albedo_factor: Vector3<f32>,
        metallic: TextureDescriptor,
        metallic_factor: f32,
        roughness: TextureDescriptor,
        roughness_factor: f32,
        occlusion: TextureDescriptor,
        emissive: TextureDescriptor,
        custom_shader: ShaderDescriptor,
    },
    WorldEnvironment(WorldEnvironmentDescriptor),
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
