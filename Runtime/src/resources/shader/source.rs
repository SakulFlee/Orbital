use std::fs::read_to_string;

use super::ShaderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderSource {
    Path(&'static str),
    String(&'static str),
}

impl Default for ShaderSource {
    fn default() -> Self {
        Self::String("#import <default>")
    }
}

impl ShaderSource {
    pub fn read_as_string(self) -> Result<String, ShaderError> {
        match self {
            ShaderSource::Path(path) => read_to_string(path).map_err(ShaderError::IO),
            ShaderSource::String(string) => Ok(string.to_string()),
        }
    }
}
