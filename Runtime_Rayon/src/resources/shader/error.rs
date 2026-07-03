use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    io::Error as IOError,
};

use crate::{resources::TextureError, shader_preprocessor::ShaderPreprocessorError};

#[derive(Debug)]
pub enum ShaderError {
    ShaderPreprocessor(ShaderPreprocessorError),
    Texture(TextureError),
    IO(IOError),
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{self:?}")
    }
}

impl Error for ShaderError {}
