use std::io::{Cursor, Read};

use cgmath::Vector2;
use log::debug;
use orbital_file_manager::FileManager;
use serde::{Deserialize, Serialize};
use wgpu::{Device, Queue, TextureFormat, TextureUsages};

use crate::{Texture as OrbitalTexture, WorldEnvironmentDescriptor, WorldEnvironmentError};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFile {
    pub ibl_diffuse_data: Vec<u8>,
    pub ibl_specular_data: Vec<u8>,
    /// The actual number of mip levels generated for the specular IBL texture.
    /// This is stored in the cache to ensure correct texture creation on load,
    /// regardless of the `WorldEnvironmentDescriptor` used for loading.
    pub ibl_specular_mip_level_count: u32,
}

impl CacheFile {
    pub fn from_path(path: &str) -> Result<Self, WorldEnvironmentError> {
        let file_manager = FileManager::global().map_err(WorldEnvironmentError::Fs)?;
        let bytes = file_manager
            .read_cache_bytes(path)
            .map_err(WorldEnvironmentError::Fs)?;
        Self::from_bytes(&bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, WorldEnvironmentError> {
        let mut reader = Cursor::new(bytes);

        // Read sizes
        let mut size_buffer = [0u8; 8];
        reader
            .read_exact(&mut size_buffer)
            .map_err(WorldEnvironmentError::IO)?;
        let diffuse_size = u64::from_le_bytes(size_buffer);
        debug!("IBL Diffuse expected size in bytes: {diffuse_size}");

        reader
            .read_exact(&mut size_buffer)
            .map_err(WorldEnvironmentError::IO)?;
        let specular_size = u64::from_le_bytes(size_buffer);
        debug!("IBL Specular expected size in bytes: {specular_size}");

        // Read specular mip level count
        let mut mip_level_buffer = [0u8; 4];
        reader
            .read_exact(&mut mip_level_buffer)
            .map_err(WorldEnvironmentError::IO)?;
        let ibl_specular_mip_level_count = u32::from_le_bytes(mip_level_buffer);
        log::debug!("IBL Specular mip level count read from cache: {ibl_specular_mip_level_count}");

        // Read data
        let mut ibl_diffuse_data = vec![0u8; diffuse_size as usize];
        let mut ibl_specular_data = vec![0u8; specular_size as usize];

        reader
            .read_exact(&mut ibl_diffuse_data)
            .map_err(WorldEnvironmentError::IO)?;
        reader
            .read_exact(&mut ibl_specular_data)
            .map_err(WorldEnvironmentError::IO)?;

        Ok(Self {
            ibl_diffuse_data,
            ibl_specular_data,
            ibl_specular_mip_level_count,
        })
    }

    pub fn to_path(&self, path: &str) -> Result<(), WorldEnvironmentError> {
        let file_manager = FileManager::global().map_err(WorldEnvironmentError::Fs)?;
        file_manager
            .write_cache_bytes(path, &self.to_bytes())
            .map_err(WorldEnvironmentError::Fs)
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            8 + 8 + 4 + self.ibl_diffuse_data.len() + self.ibl_specular_data.len(),
        );

        // Write sizes first
        bytes.extend_from_slice(&(self.ibl_diffuse_data.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.ibl_specular_data.len() as u64).to_le_bytes());

        // Write specular mip level count
        bytes.extend_from_slice(&self.ibl_specular_mip_level_count.to_le_bytes());

        // Write actual data
        bytes.extend_from_slice(&self.ibl_diffuse_data);
        bytes.extend_from_slice(&self.ibl_specular_data);

        bytes
    }

    pub fn make_textures(
        &self,
        world_environment_descriptor: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> (OrbitalTexture, OrbitalTexture) {
        let cube_face_size = match world_environment_descriptor {
            WorldEnvironmentDescriptor::FromFile { cube_face_size, .. } => *cube_face_size,
            WorldEnvironmentDescriptor::FromData { cube_face_size, .. } => *cube_face_size,
            WorldEnvironmentDescriptor::Generated { cube_face_size, .. } => *cube_face_size,
            WorldEnvironmentDescriptor::None => {
                panic!("CacheFile::make_textures called with WorldEnvironmentDescriptor::None")
            }
        };

        // Use the cached mip level count for the specular texture to ensure
        // consistency between generation and loading.
        let specular_mip_level = self.ibl_specular_mip_level_count;

        let ibl_diffuse_texture = OrbitalTexture::from_binary_data(
            &self.ibl_diffuse_data,
            Some("PBR IBL Diffuse"),
            Vector2 {
                x: cube_face_size,
                y: cube_face_size,
            },
            TextureFormat::Rgba16Float,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            1, // Diffuse IBL always has 1 mip level
            device,
            queue,
        );
        let ibl_specular_texture = OrbitalTexture::from_binary_data(
            &self.ibl_specular_data,
            Some("PBR IBL Specular"),
            Vector2 {
                x: cube_face_size,
                y: cube_face_size,
            },
            TextureFormat::Rgba16Float,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            specular_mip_level, // Use the cached mip level count
            device,
            queue,
        );

        (ibl_diffuse_texture, ibl_specular_texture)
    }

    pub fn validate(&self) -> bool {
        !self.ibl_diffuse_data.is_empty() &&
            // Check if IBL Specular's exist
            !self.ibl_specular_data.is_empty()
    }
}
