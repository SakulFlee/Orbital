use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

use cgmath::Vector2;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use wgpu::{Device, Queue, TextureFormat, TextureUsages};

use crate::resources::{
    Texture as OrbitalTexture, WorldEnvironmentDescriptor, WorldEnvironmentError,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFile {
    pub ibl_diffuse_data: Vec<u8>,
    pub ibl_specular_data: Vec<u8>,
}

impl CacheFile {
    pub fn from_path<P>(path: P) -> Result<Self, WorldEnvironmentError>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path).map_err(WorldEnvironmentError::IO)?;

        // Read sizes
        let mut size_buffer = [0u8; 8];
        file.read_exact(&mut size_buffer)
            .map_err(WorldEnvironmentError::IO)?;
        let diffuse_size = u64::from_le_bytes(size_buffer);
        debug!("IBL Diffuse expected size in bytes: {diffuse_size}");

        file.read_exact(&mut size_buffer)
            .map_err(WorldEnvironmentError::IO)?;
        let specular_size = u64::from_le_bytes(size_buffer);
        debug!("IBL Specular expected size in bytes: {specular_size}");

        // Read data
        let mut ibl_diffuse_data = vec![0u8; diffuse_size as usize];
        let mut ibl_specular_data = vec![0u8; specular_size as usize];

        file.read_exact(&mut ibl_diffuse_data)
            .map_err(WorldEnvironmentError::IO)?;
        file.read_exact(&mut ibl_specular_data)
            .map_err(WorldEnvironmentError::IO)?;

        Ok(Self {
            ibl_diffuse_data,
            ibl_specular_data,
        })
    }

    pub fn to_path<P>(&self, path: P) -> Result<(), WorldEnvironmentError>
    where
        P: AsRef<Path>,
    {
        // Create parent folder(s) if they don't exist
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(WorldEnvironmentError::IO)?;
            }
        } else {
            warn!(
                "Path doesn't have a parent, the next step might fail to save the cache to disk!"
            );
        }

        // Create the file if it doesn't exist, truncate if it does, and write self to it
        let mut file = File::create(path).map_err(WorldEnvironmentError::IO)?;

        // Write sizes first
        file.write_all(&(self.ibl_diffuse_data.len() as u64).to_le_bytes())
            .map_err(WorldEnvironmentError::IO)?;
        file.write_all(&(self.ibl_specular_data.len() as u64).to_le_bytes())
            .map_err(WorldEnvironmentError::IO)?;

        // Write actual data
        file.write_all(&self.ibl_diffuse_data)
            .map_err(WorldEnvironmentError::IO)?;
        file.write_all(&self.ibl_specular_data)
            .map_err(WorldEnvironmentError::IO)?;

        file.flush().map_err(WorldEnvironmentError::IO)?;

        Ok(())
    }

    pub fn make_textures(
        &self,
        world_environment_descriptor: &WorldEnvironmentDescriptor,
        device: &Device,
        queue: &Queue,
    ) -> (OrbitalTexture, OrbitalTexture) {
        let (cube_face_size, specular_mip_level_count) = match world_environment_descriptor {
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size,
                path: _,
                sampling_type: _,
                custom_specular_mip_level_count: specular_mip_level_count,
            } => (*cube_face_size, specular_mip_level_count.unwrap_or(1)),
            WorldEnvironmentDescriptor::FromData {
                cube_face_size,
                data: _,
                size: _,
                sampling_type: _,
                specular_mip_level_count,
            } => (*cube_face_size, specular_mip_level_count.unwrap_or(1)),
        };

        let ibl_diffuse_texture = OrbitalTexture::from_binary_data(
            &self.ibl_diffuse_data,
            Some("PBR IBL Diffuse"),
            Vector2 {
                x: cube_face_size,
                y: cube_face_size,
            },
            TextureFormat::Rgba16Float,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            1,
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
            specular_mip_level_count,
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
