use cgmath::Vector4;
use log::debug;

use super::{ShaderDescriptor, TextureDescriptor};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MaterialDescriptor {
    /// Creates a standard PBR (= Physically-Based-Rendering) material.
    PBR {
        normal: TextureDescriptor,
        albedo: TextureDescriptor,
        metallic: TextureDescriptor,
        roughness: TextureDescriptor,
    },
    /// Creates a PBR (= Physically-Based-Rendering) material
    /// with a custom shader.
    PBRCustomShader {
        normal: TextureDescriptor,
        albedo: TextureDescriptor,
        metallic: TextureDescriptor,
        roughness: TextureDescriptor,
        custom_shader: ShaderDescriptor,
    },
}

impl From<&easy_gltf::Material> for MaterialDescriptor {
    fn from(value: &easy_gltf::Material) -> Self {
        let normal = if let Some(x) = &value.normal {
            let mut processed_bytes = Vec::new();
            for (k, v) in x.texture.to_vec().into_iter().enumerate() {
                processed_bytes.push(v);
                if k % 3 == 0 {
                    processed_bytes.push(255);
                }
            }

            TextureDescriptor::StandardSRGBAu8Data(processed_bytes, x.texture.dimensions().into())
        } else {
            TextureDescriptor::UNIFORM_BLACK
        };

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

        Self::PBR {
            normal,
            albedo: albedo_texture_descriptor,
            metallic: metallic_texture_descriptor,
            roughness: roughness_texture_descriptor,
        }
    }
}
