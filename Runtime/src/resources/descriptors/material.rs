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
    /// Should only be used in rare cases and internally.
    /// Used to tag [Material](crate::resources::realizations::Material)'s in
    /// case they got created in a custom way or loaded from e.g. a glTF file.
    Tag(String),
}
