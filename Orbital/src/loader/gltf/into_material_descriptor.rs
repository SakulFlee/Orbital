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
                    rgb_to_rgba(x.as_raw()),
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
