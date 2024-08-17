use std::{
    io::{Cursor, Read},
    sync::Arc,
};

use cgmath::{Vector2, Vector3, Vector4};
use image::{ImageBuffer, ImageEncoder, ImageError, ImageFormat, Pixel, Rgba};
use log::{error, warn};
use wgpu::TextureFormat;

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
                path: "Assets/HDRs/brown_photostudio_02_4k.hdr",
            },
            irradiance: CubeTextureDescriptor::RadianceHDRFile {
                cube_face_size: 512,
                path: "Assets/HDRs/test_irradiance.hdr",
            },
            radiance: CubeTextureDescriptor::RadianceHDRFile {
                cube_face_size: 512,
                path: "Assets/HDRs/test_radiance.hdr",
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

    fn load_texture_buffer_rgb_or_rgba_fallible(
        data: &[u8],
        size: Vector2<u32>,
        format: ImageFormat,
    ) -> Result<TextureDescriptor, ImageError> {
        match image::load_from_memory_with_format(data, format) {
            Ok(dynamic_img) => {
                let rgba_img = dynamic_img.to_rgba8();
                let rgba_data = rgba_img.to_vec();
                Ok(TextureDescriptor::StandardSRGBAu8Data(rgba_data, size))
            }
            Err(e) => Err(e),
        }
    }

    fn load_texture_buffer_rgb_or_rgba(
        texture: Option<(&Vec<u8>, Vector2<u32>)>,
        factor: Option<Vector3<f32>>,
        formats: Vec<ImageFormat>,
    ) -> TextureDescriptor {
        if let Some((data, size)) = texture {
            for format in formats {
                match Self::load_texture_buffer_rgb_or_rgba_fallible(data, size, format) {
                    Ok(x) => return x,
                    Err(_) => (),
                }
            }
        }

        if let Some(factor) = factor {
            warn!("Could not load texture from memory with multiple formats. Using Uniform factor '{:?}' instead!", factor);
            Self::uniform_texture_rgb(factor)
        } else {
            warn!("Could not load texture from memory with multiple formats. Using Uniform black instead!");
            TextureDescriptor::UNIFORM_BLACK
        }
    }

    fn uniform_texture_rgb(factor: Vector3<f32>) -> TextureDescriptor {
        TextureDescriptor::UniformColor(Vector4 {
            x: (factor.x * 255.0) as u8,
            y: (factor.y * 255.0) as u8,
            z: (factor.z * 255.0) as u8,
            w: 255,
        })
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
            .ok_or(TextureDescriptor::UNIFORM_BLACK)
            .unwrap();

        let albedo = value
            .pbr
            .base_color_texture
            .as_ref()
            .map(|x| {
                TextureDescriptor::StandardSRGBAu8Data(x.as_raw().to_vec(), x.dimensions().into())
            })
            .ok_or(TextureDescriptor::UniformColor(
                value.pbr.base_color_factor.map(|x| (x * 255.0) as u8),
            ))
            .unwrap();

        let metallic = value
            .pbr
            .metallic_texture
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.as_raw().to_vec(),
                size: x.dimensions().into(),
            })
            .ok_or(TextureDescriptor::UniformLuma {
                data: (value.pbr.metallic_factor * 255.0) as u8,
            })
            .unwrap();

        let roughness = value
            .pbr
            .roughness_texture
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.as_raw().to_vec(),
                size: x.dimensions().into(),
            })
            .ok_or(TextureDescriptor::UniformLuma {
                data: (value.pbr.roughness_factor * 255.0) as u8,
            })
            .unwrap();

        let occlusion = value
            .occlusion
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.texture.to_vec(),
                size: x.texture.dimensions().into(),
            })
            .ok_or(TextureDescriptor::UNIFORM_LUMA_BLACK)
            .unwrap();

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
            .ok_or(TextureDescriptor::UniformColor(Vector4 {
                x: (value.emissive.factor.x * 255.0) as u8,
                y: (value.emissive.factor.y * 255.0) as u8,
                z: (value.emissive.factor.z * 255.0) as u8,
                w: 255,
            }))
            .unwrap();

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
