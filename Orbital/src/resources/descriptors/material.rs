use log::{error, warn};

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
        } = gltf_material.into()
        {
            Self::PBRCustomShader {
                normal,
                albedo,
                metallic,
                roughness,
                occlusion,
                custom_shader,
            }
        } else {
            unreachable!()
        }
    }
}

impl From<&easy_gltf::Material> for MaterialDescriptor {
    fn from(value: &easy_gltf::Material) -> Self {
        let normal = match &value.normal {
            Some(normal_map) => match image::load_from_memory(&normal_map.texture.as_raw()) {
                Ok(x) => {
                    let rgba_img = x.to_rgba8();
                    let bytes_vec = rgba_img.to_vec();
                    TextureDescriptor::StandardSRGBAu8Data(bytes_vec, rgba_img.dimensions().into())
                }
                Err(e) => {
                    error!(
                        "Failed loading NormalMap: {}! Using Uniform black instead.",
                        e
                    );
                    TextureDescriptor::UNIFORM_BLACK
                }
            },
            None => {
                warn!("No Normal texture found! Using uniform black.");
                TextureDescriptor::UNIFORM_BLACK
            }
        };

        let albedo = match &value.pbr.base_color_texture {
            Some(albedo_buffer) => match image::load_from_memory(&albedo_buffer.as_raw()) {
                Ok(dynamic_img) => {
                    let rgba_img = dynamic_img.to_rgba8();
                    let bytes_vec = rgba_img.to_vec();
                    TextureDescriptor::StandardSRGBAu8Data(bytes_vec, rgba_img.dimensions().into())
                }
                Err(e) => {
                    error!(
                        "Failed loading NormalMap: {}! Using Uniform black instead.",
                        e
                    );
                    TextureDescriptor::UNIFORM_BLACK
                }
            },
            None => {
                warn!("No Albedo texture found! Using uniform black.");
                TextureDescriptor::UNIFORM_BLACK
            }
        };

        let metallic = match &value.pbr.metallic_texture {
            Some(img_buffer) => TextureDescriptor::Luma {
                data: img_buffer.to_vec(),
                size: img_buffer.dimensions().into(),
            },
            None => {
                warn!("No Metallic texture found! Using uniform black.");
                TextureDescriptor::UNIFORM_LUMA_BLACK
            }
        };

        let roughness = match &value.pbr.roughness_texture {
            Some(img_buffer) => TextureDescriptor::Luma {
                data: img_buffer.to_vec(),
                size: img_buffer.dimensions().into(),
            },
            None => {
                warn!("No Roughness texture found! Using uniform black.");
                TextureDescriptor::UNIFORM_LUMA_BLACK
            }
        };

        let occlusion = match &value.occlusion {
            Some(occlusion) => TextureDescriptor::Luma {
                data: occlusion.texture.to_vec(),
                size: occlusion.texture.dimensions().into(),
            },
            None => {
                warn!("No Occlusion texture found! Using uniform black.");
                TextureDescriptor::UNIFORM_LUMA_BLACK
            }
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
