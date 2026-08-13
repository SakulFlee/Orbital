use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    io::Error as IOError,
};

use crate::TextureError;
use orbital_shader_preprocessor::ShaderPreprocessorError;

#[derive(Debug)]
pub enum ShaderError {
    ShaderPreprocessor(ShaderPreprocessorError),
    Texture(TextureError),
    IO(IOError),
    Fs(orbital_file_manager::FsError),
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{self:?}")
    }
}

impl Error for ShaderError {}
