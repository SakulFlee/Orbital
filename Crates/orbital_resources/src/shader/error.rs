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
    VariableTypeMismatch {
        binding: u32,
        expected: &'static str,
    },
    MissingVariable {
        binding: u32,
    },
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::ShaderPreprocessor(e) => write!(f, "Shader preprocessor error: {e}"),
            Self::Texture(e) => write!(f, "Texture error: {e}"),
            Self::IO(e) => write!(f, "IO error: {e}"),
            Self::VariableTypeMismatch { binding, expected } => {
                write!(
                    f,
                    "Variable at binding {binding} is not the expected type ({expected})"
                )
            }
            Self::MissingVariable { binding } => {
                write!(
                    f,
                    "Expected variable at binding {binding} but it was not found"
                )
            }
        }
    }
}

impl Error for ShaderError {}
