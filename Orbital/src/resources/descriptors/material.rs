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

    fn rgb_to_rgba(data: &[u8]) -> Vec<u8> {
        data.chunks(3)
            .map(|x| [x[0], x[1], x[2], 255])
            .collect::<Vec<_>>()
            .concat()
    }
}

impl From<&easy_gltf::Material> for MaterialDescriptor {
    fn from(value: &easy_gltf::Material) -> Self {
        let normal = value
            .normal
            .as_ref()
            .map(|x| {
                let data = Self::rgb_to_rgba(&x.texture);
                TextureDescriptor::StandardSRGBAu8Data(data, x.texture.dimensions().into())
            })
            .unwrap_or(TextureDescriptor::UNIFORM_BLACK);

        let albedo = value
            .pbr
            .base_color_texture
            .as_ref()
            .map(|x| {
                TextureDescriptor::StandardSRGBAu8Data(x.as_raw().to_vec(), x.dimensions().into())
            })
            .unwrap_or(TextureDescriptor::UniformColor(
                value.pbr.base_color_factor.map(|x| (x * 255.0) as u8),
            ));

        let metallic = value
            .pbr
            .metallic_texture
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.as_raw().to_vec(),
                size: x.dimensions().into(),
            })
            .unwrap_or(TextureDescriptor::UniformLuma {
                data: (value.pbr.metallic_factor * 255.0) as u8,
            });

        let roughness = value
            .pbr
            .roughness_texture
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.as_raw().to_vec(),
                size: x.dimensions().into(),
            })
            .unwrap_or(TextureDescriptor::UniformLuma {
                data: (value.pbr.roughness_factor * 255.0) as u8,
            });

        let occlusion = value
            .occlusion
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.texture.to_vec(),
                size: x.texture.dimensions().into(),
            })
            .unwrap_or(TextureDescriptor::UNIFORM_LUMA_WHITE);

        let emissive = value
            .emissive
            .texture
            .as_ref()
            .map(|x| {
                TextureDescriptor::StandardSRGBAu8Data(
                    Self::rgb_to_rgba(x.as_raw()),
                    x.dimensions().into(),
                )
            })
            .unwrap_or(TextureDescriptor::UNIFORM_WHITE);

        // TODO: Include factors!
        // Factors != Values

        Self::PBR {
            normal,
            albedo,
            metallic,
            roughness,
            occlusion,
            emissive,
        }
    }
}
