use cgmath::Vector4;

use super::{ShaderDescriptor, TextureDescriptor};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MaterialDescriptor {
    /// Creates a standard PBR (= Physically-Based-Rendering) material.
    PBR {
        albedo: TextureDescriptor,
        metallic: TextureDescriptor,
        roughness: TextureDescriptor,
    },
    /// Creates a PBR (= Physically-Based-Rendering) material
    /// with a custom shader.
    PBRCustomShader {
        albedo: TextureDescriptor,
        metallic: TextureDescriptor,
        roughness: TextureDescriptor,
        custom_shader: ShaderDescriptor,
    },
}

impl From<&easy_gltf::Material> for MaterialDescriptor {
    fn from(value: &easy_gltf::Material) -> Self {
        let metallic_texture_descriptor = if let Some(metallic_buffer) = &value.pbr.metallic_texture
        {
            TextureDescriptor::Luma {
                data: metallic_buffer.to_vec(),
                size: metallic_buffer.dimensions().into(),
            }
        } else {
            TextureDescriptor::UniformLuma {
                data: (255f32 * value.pbr.metallic_factor) as u8,
            }
        };

        let roughness_texture_descriptor =
            if let Some(roughness_buffer) = &value.pbr.roughness_texture {
                TextureDescriptor::Luma {
                    data: roughness_buffer.to_vec(),
                    size: roughness_buffer.dimensions().into(),
                }
            } else {
                TextureDescriptor::UniformLuma {
                    data: (255f32 * value.pbr.roughness_factor) as u8,
                }
            };

        let albedo_texture_descriptor = if let Some(base_color) = &value.pbr.base_color_texture {
            TextureDescriptor::StandardSRGBu8Data(
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

        Self::PBR {
            albedo: albedo_texture_descriptor,
            metallic: metallic_texture_descriptor,
            roughness: roughness_texture_descriptor,
        }
    }
}
