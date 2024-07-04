use super::{ShaderDescriptor, TextureDescriptor};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MaterialDescriptor {
    /// Creates a standard PBR (= Physically-Based-Rendering) material.
    ///
    /// # Parameters
    ///
    /// 1. Albedo Texture (Color)
    PBR(TextureDescriptor),
    /// Creates a PBR (= Physically-Based-Rendering) material
    /// with a custom shader.
    ///
    /// # Parameters
    ///
    /// 1. Albedo Texture (Color)
    /// 2. Custom Shader
    PBRCustomShader(TextureDescriptor, ShaderDescriptor),
}

impl From<&easy_gltf::Material> for MaterialDescriptor {
    fn from(value: &easy_gltf::Material) -> Self {
        let albedo_texture_descriptor = if let Some(base_color) = &value.pbr.base_color_texture {
            TextureDescriptor::StandardSRGBu8Data(
                base_color.to_vec(),
                base_color.dimensions().into(),
            )
        } else {
            TextureDescriptor::UNIFORM_GRAY
        };

        Self::PBR(albedo_texture_descriptor)
    }
}
