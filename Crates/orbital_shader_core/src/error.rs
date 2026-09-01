use orbital_shader_preprocessor::ShaderPreprocessorError;
use orbital_texture::TextureError;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug)]
pub enum ShaderError {
    ShaderPreprocessor(ShaderPreprocessorError),
    Texture(TextureError),
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{self:?}")
    }
}

impl Error for ShaderError {}
