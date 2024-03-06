use std::path::Path;

use wgpu::TextureFormat;

use crate::engine::{EngineResult, LogicalDevice, ResourceManager};

use super::DiffuseTexture;

pub type NormalTexture = DiffuseTexture;

pub const NORMAL_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

impl ResourceManager {
    pub fn normal_texture_from_path<P>(
        logical_device: &LogicalDevice,
        file_path: P,
    ) -> EngineResult<DiffuseTexture>
    where
        P: AsRef<Path>,
    {
        NormalTexture::from_path(logical_device, file_path, Some(NORMAL_TEXTURE_FORMAT))
    }
}
