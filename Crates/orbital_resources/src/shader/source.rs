use orbital_file_manager::FileManager;

use super::ShaderError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShaderSource {
    Path(&'static str),
    String(&'static str),
}

impl Default for ShaderSource {
    fn default() -> Self {
        Self::String(include_str!("default_shader.wgsl"))
    }
}

impl ShaderSource {
    pub fn read_as_string(self) -> Result<String, ShaderError> {
        match self {
            ShaderSource::Path(path) => {
                let file_manager = FileManager::global().map_err(ShaderError::Fs)?;
                file_manager
                    .read_asset_to_string(path)
                    .map_err(ShaderError::Fs)
            }
            ShaderSource::String(string) => Ok(string.to_string()),
        }
    }
}
