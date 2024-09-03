use super::{CubeTextureDescriptor, ShaderDescriptor, TextureDescriptor};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MaterialDescriptor {
    /// Creates a standard PBR (= Physically-Based-Rendering) material.
    PBR {
        normal: TextureDescriptor,
        albedo: TextureDescriptor,
        metallic: TextureDescriptor,
        roughness: TextureDescriptor,
        occlusion: TextureDescriptor,
        emissive: TextureDescriptor,
    },
    /// Creates a PBR (= Physically-Based-Rendering) material
    /// with a custom shader.
    PBRCustomShader {
        normal: TextureDescriptor,
        albedo: TextureDescriptor,
        metallic: TextureDescriptor,
        roughness: TextureDescriptor,
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

    pub fn from_gltf(gltf_material: &easy_gltf::Material) -> Self {
        gltf_material.into()
    }

    pub fn from_gltf_with_custom_shader(
        gltf_material: &easy_gltf::Material,
        custom_shader: ShaderDescriptor,
    ) -> Self {
        if let Self::PBR {
            normal,
            albedo,
            metallic,
            roughness,
            occlusion,
            emissive,
        } = gltf_material.into()
        {
            Self::PBRCustomShader {
                normal,
                albedo,
                metallic,
                roughness,
                occlusion,
                emissive,
                custom_shader,
            }
        } else {
            unreachable!()
        }
    }
}
