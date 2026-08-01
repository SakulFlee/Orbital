use std::hash::{Hash, Hasher};

use cgmath::Vector2;

use super::SamplingType;

#[derive(Debug, Clone)]
pub struct GeneratedSkyParameters {
    /// Time of day in hours (0.0 = midnight, 12.0 = noon, 24.0 = midnight).
    /// Drives the sun elevation along a daily arc. Default: 14.0 (2 PM).
    pub time_of_day_hours: f32,
    /// Azimuth of the sun in radians (0.0 = along +X, rising/setting arc in
    /// the XZ plane). Default: 0.0.
    pub sun_azimuth: f32,
    /// Angular radius of the sun disk in radians (default: ~0.267° for Earth's sun).
    pub sun_angular_radius: f32,
    /// Sun intensity multiplier (default: 20.0).
    pub sun_intensity: f32,
    /// Angular radius of the moon disk in radians (default: ~0.46°).
    pub moon_angular_radius: f32,
    /// Moon intensity multiplier (default: 2.0).
    pub moon_intensity: f32,
    /// Star intensity multiplier (0.0 = no stars, 1.0 = default). Default: 1.0.
    pub star_intensity: f32,
    /// Ground albedo color, RGB. Used for the lower hemisphere below the
    /// horizon (default: [0.3, 0.3, 0.3]).
    pub ground_albedo: [f32; 3],
    /// Exposure multiplier applied to the final HDR output (default: 1.0).
    pub exposure: f32,
}

impl PartialEq for GeneratedSkyParameters {
    fn eq(&self, other: &Self) -> bool {
        fn f3_eq(a: &[f32; 3], b: &[f32; 3]) -> bool {
            a.iter()
                .zip(b.iter())
                .all(|(x, y)| x.to_bits() == y.to_bits())
        }

        self.time_of_day_hours.to_bits() == other.time_of_day_hours.to_bits()
            && self.sun_azimuth.to_bits() == other.sun_azimuth.to_bits()
            && self.sun_angular_radius.to_bits() == other.sun_angular_radius.to_bits()
            && self.sun_intensity.to_bits() == other.sun_intensity.to_bits()
            && self.moon_angular_radius.to_bits() == other.moon_angular_radius.to_bits()
            && self.moon_intensity.to_bits() == other.moon_intensity.to_bits()
            && self.star_intensity.to_bits() == other.star_intensity.to_bits()
            && f3_eq(&self.ground_albedo, &other.ground_albedo)
            && self.exposure.to_bits() == other.exposure.to_bits()
    }
}

impl Eq for GeneratedSkyParameters {}

impl Hash for GeneratedSkyParameters {
    fn hash<H: Hasher>(&self, state: &mut H) {
        fn hash_f3<H: Hasher>(s: &mut H, v: &[f32; 3]) {
            for x in v {
                x.to_bits().hash(s);
            }
        }

        self.time_of_day_hours.to_bits().hash(state);
        self.sun_azimuth.to_bits().hash(state);
        self.sun_angular_radius.to_bits().hash(state);
        self.sun_intensity.to_bits().hash(state);
        self.moon_angular_radius.to_bits().hash(state);
        self.moon_intensity.to_bits().hash(state);
        self.star_intensity.to_bits().hash(state);
        hash_f3(state, &self.ground_albedo);
        self.exposure.to_bits().hash(state);
    }
}

#[derive(Debug, Clone, Eq)]
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
    /// Procedurally generate the environment map using an analytic
    /// time-of-day sky. Produces an equirectangular HDR sky texture and
    /// converts it into diffuse (irradiance) and specular (radiance) cube
    /// textures — just like loading an HDRI file, but entirely generated on
    /// the GPU.
    ///
    /// The sky is driven by [`GeneratedSkyParameters::time_of_day_hours`]:
    /// blue sky with a bright sun during the day, warm twilight at dawn and
    /// dusk, and a dark starry night (with a moon) after sunset.
    ///
    /// This is the **default** when no descriptor is set.
    Generated {
        cube_face_size: u32,
        sampling_type: SamplingType,
        custom_specular_mip_level_count: Option<u32>,
        /// Sky parameters. If `None`, uses [`GeneratedSkyParameters::default()`].
        parameters: Option<GeneratedSkyParameters>,
    },
    /// Explicitly disable the environment — no skybox, no IBL lighting.
    None,
}

impl Default for GeneratedSkyParameters {
    fn default() -> Self {
        Self {
            time_of_day_hours: 14.0,
            sun_azimuth: 0.0,
            sun_angular_radius: 0.004_65,
            sun_intensity: 20.0,
            moon_angular_radius: 0.008,
            moon_intensity: 2.0,
            star_intensity: 1.0,
            ground_albedo: [0.3, 0.3, 0.3],
            exposure: 1.0,
        }
    }
}

impl WorldEnvironmentDescriptor {
    pub const DEFAULT_SIZE: u32 = 2048;
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
                if !(l_cube_face_size == r_cube_face_size
                    && l_size == r_size
                    && l_sampling_type == r_sampling_type
                    && l_specular_mip_level_count == r_specular_mip_level_count)
                {
                    return false;
                }

                l_data.iter().zip(r_data.iter()).any(|(l, r)| l.eq(r))
            }
            (
                Self::Generated {
                    cube_face_size: l_cube_face_size,
                    sampling_type: l_sampling_type,
                    custom_specular_mip_level_count: l_specular_mip_level_count,
                    parameters: l_parameters,
                },
                Self::Generated {
                    cube_face_size: r_cube_face_size,
                    sampling_type: r_sampling_type,
                    custom_specular_mip_level_count: r_specular_mip_level_count,
                    parameters: r_parameters,
                },
            ) => {
                l_cube_face_size == r_cube_face_size
                    && l_sampling_type == r_sampling_type
                    && l_specular_mip_level_count == r_specular_mip_level_count
                    && l_parameters == r_parameters
            }
            (Self::None, Self::None) => true,
            _ => false,
        }
    }
}

impl Hash for WorldEnvironmentDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size,
                path,
                sampling_type,
                custom_specular_mip_level_count,
            } => {
                cube_face_size.hash(state);
                path.hash(state);
                sampling_type.hash(state);
                custom_specular_mip_level_count.hash(state);
            }
            WorldEnvironmentDescriptor::FromData {
                cube_face_size,
                data,
                size,
                sampling_type,
                specular_mip_level_count,
            } => {
                cube_face_size.hash(state);
                data.hash(state);
                size.hash(state);
                sampling_type.hash(state);
                specular_mip_level_count.hash(state);
            }
            WorldEnvironmentDescriptor::Generated {
                cube_face_size,
                sampling_type,
                custom_specular_mip_level_count,
                parameters,
            } => {
                cube_face_size.hash(state);
                sampling_type.hash(state);
                custom_specular_mip_level_count.hash(state);
                parameters.hash(state);
            }
            WorldEnvironmentDescriptor::None => {
                0u8.hash(state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_sky_params_default_equals_default() {
        let a = GeneratedSkyParameters::default();
        let b = GeneratedSkyParameters::default();
        assert_eq!(a, b);
    }

    #[test]
    fn generated_sky_params_default_hashes_consistently() {
        let a = GeneratedSkyParameters::default();
        let b = GeneratedSkyParameters::default();
        let mut h1 = std::hash::DefaultHasher::new();
        let mut h2 = std::hash::DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn generated_descriptor_hash_consistent_for_cache() {
        let a = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: None,
        };
        let b = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: None,
        };
        assert_eq!(a, b);

        let mut h1 = std::hash::DefaultHasher::new();
        let mut h2 = std::hash::DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn generated_descriptor_hash_differs_with_params() {
        let a = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: Some(GeneratedSkyParameters::default()),
        };
        let mut params = GeneratedSkyParameters::default();
        params.exposure = 2.0;
        let b = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: Some(params),
        };
        assert_ne!(a, b);

        let mut h1 = std::hash::DefaultHasher::new();
        let mut h2 = std::hash::DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn none_descriptor_equals_none() {
        assert_eq!(
            WorldEnvironmentDescriptor::None,
            WorldEnvironmentDescriptor::None
        );
    }

    #[test]
    fn none_neq_generated() {
        assert_ne!(
            WorldEnvironmentDescriptor::None,
            WorldEnvironmentDescriptor::Generated {
                cube_face_size: 2048,
                sampling_type: SamplingType::ImportanceSampling,
                custom_specular_mip_level_count: None,
                parameters: None,
            }
        );
    }

    /// Verify the uniform-buffer serialization matches the WGSL `SkyParams`
    /// struct layout (3 rows × 16 bytes = 48 bytes).  Must be kept in sync
    /// with `make_sky_parameters_buffer` and `generate_sky.wgsl`.
    #[test]
    fn sky_params_wgsl_struct_is_48_bytes() {
        let p = GeneratedSkyParameters::default();
        let mut b = Vec::new();
        let w = |b: &mut Vec<u8>, f: f32| b.extend_from_slice(&f.to_le_bytes());

        // Row 0: 4 × f32
        w(&mut b, p.time_of_day_hours);
        w(&mut b, p.sun_azimuth);
        w(&mut b, p.sun_angular_radius);
        w(&mut b, p.sun_intensity);
        // Row 1: 4 × f32
        w(&mut b, p.moon_angular_radius);
        w(&mut b, p.moon_intensity);
        w(&mut b, p.star_intensity);
        w(&mut b, p.exposure);
        // Row 2: ground_albedo(vec3) + _pad2
        w(&mut b, p.ground_albedo[0]);
        w(&mut b, p.ground_albedo[1]);
        w(&mut b, p.ground_albedo[2]);
        w(&mut b, 0.0);

        assert_eq!(
            b.len(),
            48,
            "SkyParams buffer must be exactly 48 bytes (3 rows × 16 bytes)"
        );
    }
}
