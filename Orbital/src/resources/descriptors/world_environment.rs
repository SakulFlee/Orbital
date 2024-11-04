use cgmath::Vector2;

use super::{skybox_type, SkyboxType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WorldEnvironmentDescriptor {
    /// Loading an HDRI from file.  
    /// First of all, will convert the HDRI _equirectangular_ image
    /// into a _cube texture_.
    /// Secondly, will transform the _cube texture_ into a diffuse
    /// (irradiance) and specular (radiance) _cube texture_.
    FromFile {
        skybox_type: SkyboxType,
        cube_face_size: u32,
        path: &'static str,
    },
    /// Same as [WorldEnvironmentDescriptor::FromFile], but uses a data
    /// Vector instead.
    ///
    /// ⚠️ Make sure the data you supply is correct and contains an
    /// alpha channel!
    FromData {
        skybox_type: SkyboxType,
        cube_face_size: u32,
        data: Vec<u8>,
        size: Vector2<u32>,
    },
}

impl WorldEnvironmentDescriptor {
    pub const DEFAULT_SIZE: u32 = 4096;

    pub fn set_skybox_type(&mut self, new_skybox_type: SkyboxType) {
        match self {
            WorldEnvironmentDescriptor::FromFile {
                skybox_type,
                cube_face_size: _,
                path: _,
            } => {
                *skybox_type = new_skybox_type;
            }
            WorldEnvironmentDescriptor::FromData {
                skybox_type,
                cube_face_size: _,
                data: _,
                size: _,
            } => {
                *skybox_type = new_skybox_type;
            }
        }
    }
}
