use std::{sync::Arc, u8};

use cgmath::Vector3;
use easy_gltf::Material;

use crate::{
    resources::descriptors::{MaterialDescriptor, TextureDescriptor},
    util::rgb_to_rgba,
};

impl From<&Material> for MaterialDescriptor {
    fn from(value: &Material) -> Self {
        let normal = value
            .normal
            .as_ref()
            .map(|x| {
                let data = rgb_to_rgba(&x.texture);
                TextureDescriptor::StandardSRGBAu8Data(data, x.texture.dimensions().into())
            })
            .unwrap_or(TextureDescriptor::UNIFORM_BLACK);

        fn convert_factor_to_u8(factor: f32) -> u8 {
            const U8_MIN_AS_F32: f32 = u8::MIN as f32;
            const U8_MAX_AS_F32: f32 = u8::MAX as f32;

            let unclamped = factor * U8_MAX_AS_F32;
            let clamped = unclamped.clamp(U8_MIN_AS_F32, U8_MAX_AS_F32);

            clamped as u8
        }

        let albedo = value
            .pbr
            .base_color_texture
            .as_ref()
            .map(|x| {
                TextureDescriptor::StandardSRGBAu8Data(x.as_raw().to_vec(), x.dimensions().into())
            })
            .unwrap_or(TextureDescriptor::UniformColor(
                value.pbr.base_color_factor.map(convert_factor_to_u8),
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
                data: convert_factor_to_u8(value.pbr.metallic_factor),
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
                data: convert_factor_to_u8(value.pbr.roughness_factor),
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
                    rgb_to_rgba(x.as_raw()),
                    x.dimensions().into(),
                )
            })
            .unwrap_or(TextureDescriptor::UNIFORM_BLACK);

        Self::PBR {
            normal: Arc::new(normal),
            albedo: Arc::new(albedo),
            albedo_factor: Vector3::new(
                value.pbr.base_color_factor.x,
                value.pbr.base_color_factor.y,
                value.pbr.base_color_factor.z,
            ),
            metallic: Arc::new(metallic),
            metallic_factor: value.pbr.metallic_factor,
            roughness: Arc::new(roughness),
            roughness_factor: value.pbr.roughness_factor,
            occlusion: Arc::new(occlusion),
            emissive: Arc::new(emissive),
            custom_shader: None,
        }
    }
}
