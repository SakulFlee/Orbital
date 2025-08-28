use cgmath::Vector2;

use super::SamplingType;

#[derive(Debug, Clone, Hash, Eq)]
pub enum WorldEnvironmentDescriptor {
    /// Loading an HDRI from file.  
    /// First of all, will convert the HDRI _equirectangular_ image
    /// into a _cube texture_.
    /// Secondly, will transform the _cube texture_ into a diffuse
    /// (irradiance) and specular (radiance) _cube texture_.
    FromFile {
        cube_face_size: u32,
        path: String,
        sampling_type: SamplingType,
        /// Defines how many mip levels the specular texture will have.
        /// The first (index: 0) will be the base level, which is also used as the skybox.
        /// Any additional mip index will be used for reflections.
        ///
        /// With each mip level, the texture will be downsampled by a factor of 2 and thus become more blurry.
        ///
        /// A higher level here means more accurate blurry reflections, but will take a lot longer to process and uses a lot more VRAM, as well as cache space if caching is enabled.
        /// On the other hand, a lower level will give you much faster and space efficient (VRAM & cache) results, but the reflections will be less accurate.
        ///
        /// A good choice here is either 5 or 10.  
        /// 5 gives you base + 25% increments of the base level, which is a good trade-off between performance and quality.  
        /// 10 gives you base + 10% increments of the base level, which is realistically the highest you should go for.  
        /// _It might also be a good idea to check what kind of device you we are running on and adjust this value accordingly._
        ///
        /// If set to `None`, will default to 10.
        custom_specular_mip_level_count: Option<u32>,
    },
    /// Same as [WorldEnvironmentDescriptor::FromFile], but uses a data
    /// Vector instead.
    ///
    /// ⚠️ Make sure the data you supply is correct and contains an
    /// alpha channel!
    FromData {
        cube_face_size: u32,
        data: Vec<u8>,
        size: Vector2<u32>,
        sampling_type: SamplingType,
        /// Defines how many mip levels the specular texture will have.
        /// The first (index: 0) will be the base level, which is also used as the skybox.
        /// Any additional mip index will be used for reflections.
        ///
        /// With each mip level, the texture will be downsampled by a factor of 2 and thus become more blurry.
        ///
        /// A higher level here means more accurate blurry reflections, but will take a lot longer to process and uses a lot more VRAM, as well as cache space if caching is enabled.
        /// On the other hand, a lower level will give you much faster and space efficient (VRAM & cache) results, but the reflections will be less accurate.
        ///
        /// A good choice here is either 5 or 10.  
        /// 5 gives you base + 25% increments of the base level, which is a good trade-off between performance and quality.  
        /// 10 gives you base + 10% increments of the base level, which is realistically the highest you should go for.  
        /// _It might also be a good idea to check what kind of device you we are running on and adjust this value accordingly._
        ///
        /// If set to `None`, will default to 10.
        ///
        /// 10 is the **maximum** mip level count allowed by WGPU!
        specular_mip_level_count: Option<u32>,
    },
}

impl WorldEnvironmentDescriptor {
    pub const DEFAULT_SIZE: u32 = 4096;
    pub const DEFAULT_SAMPLING_TYPE: SamplingType = SamplingType::ImportanceSampling;
}

impl PartialEq for WorldEnvironmentDescriptor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::FromFile {
                    cube_face_size: l_cube_face_size,
                    path: l_path,
                    sampling_type: l_sampling_type,
                    custom_specular_mip_level_count: l_specular_mip_level_count,
                },
                Self::FromFile {
                    cube_face_size: r_cube_face_size,
                    path: r_path,
                    sampling_type: r_sampling_type,
                    custom_specular_mip_level_count: r_specular_mip_level_count,
                },
            ) => {
                l_cube_face_size == r_cube_face_size
                    && l_path == r_path
                    && l_sampling_type == r_sampling_type
                    && l_specular_mip_level_count == r_specular_mip_level_count
            }
            (
                Self::FromData {
                    cube_face_size: l_cube_face_size,
                    data: l_data,
                    size: l_size,
                    sampling_type: l_sampling_type,
                    specular_mip_level_count: l_specular_mip_level_count,
                },
                Self::FromData {
                    cube_face_size: r_cube_face_size,
                    data: r_data,
                    size: r_size,
                    sampling_type: r_sampling_type,
                    specular_mip_level_count: r_specular_mip_level_count,
                },
            ) => {
                // Check for obvious facts first.
                if !(l_cube_face_size == r_cube_face_size
                    && l_size == r_size
                    && l_sampling_type == r_sampling_type
                    && l_specular_mip_level_count == r_specular_mip_level_count)
                {
                    return false;
                }

                // Then, compare byte-by-byte with fail-fast.
                l_data.iter().zip(r_data.iter()).any(|(l, r)| l.eq(r))
            }
            _ => false,
        }
    }
}
