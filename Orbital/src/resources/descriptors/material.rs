use std::{
    hash::{Hash, Hasher},
    mem,
};

use cgmath::Vector3;

use super::{CubeTextureDescriptor, ShaderDescriptor, TextureDescriptor};

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
    WorldEnvironment {
        sky: CubeTextureDescriptor,
        irradiance: CubeTextureDescriptor,
        radiance: CubeTextureDescriptor,
    },
}

impl MaterialDescriptor {
    pub fn default_world_environment() -> MaterialDescriptor {
        MaterialDescriptor::WorldEnvironment {
            sky: CubeTextureDescriptor::RadianceHDRFile {
                cube_face_size: 1024,
                path: "Assets/HDRs/kloppenheim_02_puresky_4k.hdr",
            },
            irradiance: CubeTextureDescriptor::RadianceHDRFile {
                cube_face_size: 1024,
                path: "Assets/HDRs/kloppenheim_02_puresky_4k.hdr",
            },
            radiance: CubeTextureDescriptor::RadianceHDRFile {
                cube_face_size: 1024,
                path: "Assets/HDRs/kloppenheim_02_puresky_4k.hdr",
            },
        }
    }
}

impl Hash for MaterialDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
}

impl Eq for MaterialDescriptor {}
