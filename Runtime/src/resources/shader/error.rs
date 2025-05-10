use std::io::Error as IOError;

use crate::{resources::TextureError, shader_preprocessor::ShaderPreprocessorError};

#[derive(Debug)]
pub enum ShaderError {
    ShaderPreprocessor(ShaderPreprocessorError),
    Texture(TextureError),
    IO(IOError),
}
