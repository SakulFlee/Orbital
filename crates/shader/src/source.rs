use std::fs::read_to_string;

use crate::Error;

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
    pub fn read_as_string(self) -> Result<String, Error> {
        match self {
            ShaderSource::Path(path) => read_to_string(path).map_err(|e| Error::IO(e)),
            ShaderSource::String(string) => Ok(string.to_string()),
        }
    }
}
