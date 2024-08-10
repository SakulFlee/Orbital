use cgmath::Vector4;

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
    },
    /// Creates a PBR (= Physically-Based-Rendering) material
    /// with a custom shader.
    PBRCustomShader {
        normal: TextureDescriptor,
        albedo: TextureDescriptor,
        metallic: TextureDescriptor,
        roughness: TextureDescriptor,
        occlusion: TextureDescriptor,
        custom_shader: ShaderDescriptor,
    },
    SkyBox {
        sky_texture: CubeTextureDescriptor,
    }
}

impl MaterialDescriptor {
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
        } = gltf_material.into()
        {
            Self::PBRCustomShader {
                normal: normal,
                albedo: albedo,
                metallic: metallic,
                roughness: roughness,
                occlusion: occlusion,
                custom_shader: custom_shader,
            }
        } else {
            unreachable!()
        }
    }
}

impl From<&easy_gltf::Material> for MaterialDescriptor {
    fn from(value: &easy_gltf::Material) -> Self {
        let normal = if let Some(normal_map) = &value.normal {
            let mut processed_bytes = Vec::new();
            for (k, v) in normal_map.texture.to_vec().into_iter().enumerate() {
                processed_bytes.push(v);
                if k % 3 == 0 {
                    processed_bytes.push(255);
                }
            }

            TextureDescriptor::StandardSRGBAu8Data(
                processed_bytes,
                normal_map.texture.dimensions().into(),
            )
        } else {
            TextureDescriptor::UNIFORM_BLACK
        };

        let albedo = if let Some(base_color) = &value.pbr.base_color_texture {
            TextureDescriptor::StandardSRGBAu8Data(
                base_color.to_vec(),
                base_color.dimensions().into(),
            )
        } else {
            TextureDescriptor::UniformColor(Vector4::new(
                (255f32 * value.pbr.base_color_factor.x) as u8,
                (255f32 * value.pbr.base_color_factor.y) as u8,
                (255f32 * value.pbr.base_color_factor.z) as u8,
                (255f32 * value.pbr.base_color_factor.w) as u8,
            ))
        };

        let metallic = if let Some(metallic_buffer) = &value.pbr.metallic_texture {
            TextureDescriptor::Luma {
                data: metallic_buffer.to_vec(),
                size: metallic_buffer.dimensions().into(),
            }
        } else {
            TextureDescriptor::UniformLuma {
                data: (255f32 * value.pbr.metallic_factor) as u8,
            }
        };

        let roughness = if let Some(roughness_buffer) = &value.pbr.roughness_texture {
            TextureDescriptor::Luma {
                data: roughness_buffer.to_vec(),
                size: roughness_buffer.dimensions().into(),
            }
        } else {
            TextureDescriptor::UniformLuma {
                data: (255f32 * value.pbr.roughness_factor) as u8,
            }
        };

        let occlusion = if let Some(occlusion) = &value.occlusion {
            TextureDescriptor::Luma {
                data: occlusion.texture.to_vec(),
                size: occlusion.texture.dimensions().into(),
            }
        } else {
            TextureDescriptor::UniformLuma { data: 255u8 }
        };

        Self::PBR {
            normal,
            albedo,
            metallic,
            roughness,
            occlusion,
        }
    }
}
