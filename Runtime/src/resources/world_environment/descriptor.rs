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
        /// A good choice here is either 5 or 7.  
        /// 5 gives you the base level + 4 additional mipmap levels (5 total)
        /// 7 gives you the base level + 6 additional mipmap levels (7 total)
        /// The additional mipmap levels provide progressively blurrier reflections for materials with higher roughness values.
        ///
        /// By default, a reasonable maximum of 7 levels (the base level + 6 additional mipmap levels) is used to balance quality and performance.
        /// This prevents generating very small mipmap levels that don't contribute much to visual quality.
        ///
        /// If you need more or fewer levels, you can explicitly set this value.
        /// The maximum allowed value is determined by the cube face size (log2(size) + 1).
        ///
        /// If set to `None`, will default to 7 (or the maximum possible if less than 7).
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
        /// A good choice here is either 5 or 7.  
        /// 5 gives you the base level + 4 additional mipmap levels (5 total)
        /// 7 gives you the base level + 6 additional mipmap levels (7 total)
        /// The additional mipmap levels provide progressively blurrier reflections for materials with higher roughness values.
        ///
        /// By default, a reasonable maximum of 7 levels (the base level + 6 additional mipmap levels) is used to balance quality and performance.
        /// This prevents generating very small mipmap levels that don't contribute much to visual quality.
        ///
        /// If you need more or fewer levels, you can explicitly set this value.
        /// The maximum allowed value is determined by the cube face size (log2(size) + 1).
        ///
        /// If set to `None`, will default to 7 (or the maximum possible if less than 7).
        ///
        /// Note: The maximum mip level count is determined by the texture size (log2(size) + 1).
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
