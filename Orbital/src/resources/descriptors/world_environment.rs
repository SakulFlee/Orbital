use cgmath::Vector2;

use super::{SamplingType, SkyboxType};

#[derive(Debug, Clone, Hash, Eq)]
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
        sampling_type: SamplingType,
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
        sampling_type: SamplingType,
    },
}

impl WorldEnvironmentDescriptor {
    pub const DEFAULT_SIZE: u32 = 4096;
    pub const DEFAULT_SAMPLING_TYPE: SamplingType = SamplingType::ImportanceSampling;

    pub fn set_skybox_type(&mut self, new_skybox_type: SkyboxType) {
        match self {
            WorldEnvironmentDescriptor::FromFile {
                skybox_type,
                cube_face_size: _,
                path: _,
                sampling_type: _,
            } => {
                *skybox_type = new_skybox_type;
            }
            WorldEnvironmentDescriptor::FromData {
                skybox_type,
                cube_face_size: _,
                data: _,
                size: _,
                sampling_type: _,
            } => {
                *skybox_type = new_skybox_type;
            }
        }
    }
}

impl PartialEq for WorldEnvironmentDescriptor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::FromFile {
                    skybox_type: l_skybox_type,
                    cube_face_size: l_cube_face_size,
                    path: l_path,
                    sampling_type: l_sampling_type,
                },
                Self::FromFile {
                    skybox_type: r_skybox_type,
                    cube_face_size: r_cube_face_size,
                    path: r_path,
                    sampling_type: r_sampling_type,
                },
            ) => {
                l_skybox_type == r_skybox_type
                    && l_cube_face_size == r_cube_face_size
                    && l_path == r_path
                    && l_sampling_type == r_sampling_type
            }
            (
                Self::FromData {
                    skybox_type: l_skybox_type,
                    cube_face_size: l_cube_face_size,
                    data: l_data,
                    size: l_size,
                    sampling_type: l_sampling_type,
                },
                Self::FromData {
                    skybox_type: r_skybox_type,
                    cube_face_size: r_cube_face_size,
                    data: r_data,
                    size: r_size,
                    sampling_type: r_sampling_type,
                },
            ) => {
                // Check for obvious facts first.
                if !(l_skybox_type == r_skybox_type
                    && l_cube_face_size == r_cube_face_size
                    && l_size == r_size
                    && l_sampling_type == r_sampling_type)
                {
                    return false;
                }

                // Then, compare byte-by-byte with fail-fast.
                return l_data.iter().zip(r_data.iter()).any(|(l, r)| l.eq(r));
            }
            _ => false,
        }
    }
}
