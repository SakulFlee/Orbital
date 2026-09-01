use std::hash::{Hash, Hasher};

use cgmath::Vector2;

use super::SamplingType;

/// How the sun's elevation is controlled.
#[derive(Debug, Clone, Copy)]
pub enum SunPosition {
    /// Sun elevation driven by a 24-hour clock. `hours = 0.0` is midnight,
    /// `12.0` is noon. The sun rises at ~6 h, is overhead at ~12 h, sets at
    /// ~18 h and is below the horizon at night.
    TimeOfDay { hours: f32 },
    /// Sun elevation in degrees above the horizon: `0.0` = on the horizon,
    /// `90.0` = directly overhead, `-90.0` = directly below.
    Elevation { elevation_deg: f32 },
}

impl PartialEq for SunPosition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TimeOfDay { hours: a }, Self::TimeOfDay { hours: b }) => {
                a.to_bits() == b.to_bits()
            }
            (Self::Elevation { elevation_deg: a }, Self::Elevation { elevation_deg: b }) => {
                a.to_bits() == b.to_bits()
            }
            _ => false,
        }
    }
}

impl Eq for SunPosition {}

impl Hash for SunPosition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::TimeOfDay { hours } => {
                0u8.hash(state);
                hours.to_bits().hash(state);
            }
            Self::Elevation { elevation_deg } => {
                1u8.hash(state);
                elevation_deg.to_bits().hash(state);
            }
        }
    }
}

/// Full colour palette of the analytic sky. Every value is HDR linear RGB.
#[derive(Debug, Clone)]
pub struct SkyPalette {
    /// Sky colour directly overhead during the day (default: blue).
    pub day_zenith: [f32; 3],
    /// Sky colour near the horizon during the day (default: pale blue-white).
    pub day_horizon: [f32; 3],
    /// Sky colour directly overhead at night (default: near-black blue).
    pub night_zenith: [f32; 3],
    /// Sky colour near the horizon at night (default: dark grey-blue).
    pub night_horizon: [f32; 3],
    /// Warm colour of the twilight band near the horizon at dusk/dawn.
    pub twilight: [f32; 3],
    /// Tint of the sun disk and halo.
    pub sun_color: [f32; 3],
    /// Tint of the moon disk and halo.
    pub moon_color: [f32; 3],
}

impl PartialEq for SkyPalette {
    fn eq(&self, other: &Self) -> bool {
        fn f3_eq(a: &[f32; 3], b: &[f32; 3]) -> bool {
            a.iter()
                .zip(b.iter())
                .all(|(x, y)| x.to_bits() == y.to_bits())
        }

        f3_eq(&self.day_zenith, &other.day_zenith)
            && f3_eq(&self.day_horizon, &other.day_horizon)
            && f3_eq(&self.night_zenith, &other.night_zenith)
            && f3_eq(&self.night_horizon, &other.night_horizon)
            && f3_eq(&self.twilight, &other.twilight)
            && f3_eq(&self.sun_color, &other.sun_color)
            && f3_eq(&self.moon_color, &other.moon_color)
    }
}

impl Eq for SkyPalette {}

impl Hash for SkyPalette {
    fn hash<H: Hasher>(&self, state: &mut H) {
        fn hash_f3<H: Hasher>(s: &mut H, v: &[f32; 3]) {
            for x in v {
                x.to_bits().hash(s);
            }
        }

        hash_f3(state, &self.day_zenith);
        hash_f3(state, &self.day_horizon);
        hash_f3(state, &self.night_zenith);
        hash_f3(state, &self.night_horizon);
        hash_f3(state, &self.twilight);
        hash_f3(state, &self.sun_color);
        hash_f3(state, &self.moon_color);
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedSkyParameters {
    /// How the sun's elevation is controlled. Default: 14.0 (2 PM).
    pub sun_position: SunPosition,
    /// Azimuth of the sun in radians (0.0 = along +X, rising/setting arc in
    /// the XZ plane). Default: 0.0.
    pub sun_azimuth: f32,
    /// Angular radius of the sun disk in radians (default: ~0.02 rad ≈ 1.15°).
    /// The rendered sun is a soft Gaussian core sized by this radius, so it
    /// stays readable as a gradient rather than a sub-pixel dot.
    pub sun_angular_radius: f32,
    /// Sun intensity multiplier (default: 6.0).
    pub sun_intensity: f32,
    /// Angular radius of the moon disk in radians (default: ~0.69°).
    pub moon_angular_radius: f32,
    /// Moon intensity multiplier (default: 2.0).
    pub moon_intensity: f32,
    /// Star intensity multiplier (0.0 = no stars, 1.0 = default). Default: 1.0.
    pub star_intensity: f32,
    /// Star field density in [0.0, 1.0]. `0.0` = no stars, `1.0` = fully
    /// covered star field. Default: `0.06` (~6% of cells).
    pub star_density: f32,
    /// Ground albedo color, RGB. Used for the lower hemisphere below the
    /// horizon (default: [0.3, 0.3, 0.3]).
    pub ground_albedo: [f32; 3],
    /// Exposure multiplier applied to the final HDR output (default: 1.0).
    pub exposure: f32,
    /// Colour palette of the sky. Defaults match the built-in constants.
    pub palette: SkyPalette,
}

impl Default for GeneratedSkyParameters {
    fn default() -> Self {
        Self {
            sun_position: SunPosition::TimeOfDay { hours: 14.0 },
            sun_azimuth: 0.0,
            sun_angular_radius: 0.02,
            sun_intensity: 6.0,
            moon_angular_radius: 0.012,
            moon_intensity: 2.0,
            star_intensity: 1.0,
            star_density: 0.06,
            ground_albedo: [0.3, 0.3, 0.3],
            exposure: 1.0,
            palette: SkyPalette::default(),
        }
    }
}

impl Default for SkyPalette {
    fn default() -> Self {
        Self {
            day_zenith: [0.15, 0.4, 1.0],
            day_horizon: [0.75, 0.85, 1.0],
            night_zenith: [0.005, 0.008, 0.03],
            night_horizon: [0.02, 0.02, 0.05],
            twilight: [2.0, 0.85, 0.45],
            sun_color: [1.0, 0.92, 0.78],
            moon_color: [0.72, 0.76, 0.85],
        }
    }
}

impl GeneratedSkyParameters {
    const TWO_PI: f32 = std::f32::consts::TAU;

    /// Unit sun direction derived from [`SunPosition`] and `sun_azimuth`.
    ///
    ///   * `TimeOfDay`: `theta = (hours - 6) / 24 · 2π`; `sin(theta)` is the
    ///     elevation so 6 h → horizon, 12 h → overhead, 18 h → opposite
    ///     horizon, 0/24 h → below.
    ///   * `Elevation`: the elevation is used directly in degrees.
    pub fn sun_direction(&self) -> [f32; 3] {
        let (sin_elev, cos_elev) = match self.sun_position {
            SunPosition::TimeOfDay { hours } => {
                let theta = (hours - 6.0) / 24.0 * Self::TWO_PI;
                (theta.sin(), theta.cos())
            }
            SunPosition::Elevation { elevation_deg } => {
                let elev = elevation_deg.to_radians();
                (elev.sin(), elev.cos())
            }
        };

        let cos_az = self.sun_azimuth.cos();
        let sin_az = self.sun_azimuth.sin();
        let dir = [cos_elev * cos_az, sin_elev, cos_elev * sin_az];

        let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if len > 0.0 {
            [dir[0] / len, dir[1] / len, dir[2] / len]
        } else {
            [0.0, 1.0, 0.0]
        }
    }
}

impl PartialEq for GeneratedSkyParameters {
    fn eq(&self, other: &Self) -> bool {
        fn f3_eq(a: &[f32; 3], b: &[f32; 3]) -> bool {
            a.iter()
                .zip(b.iter())
                .all(|(x, y)| x.to_bits() == y.to_bits())
        }

        self.sun_position == other.sun_position
            && self.sun_azimuth.to_bits() == other.sun_azimuth.to_bits()
            && self.sun_angular_radius.to_bits() == other.sun_angular_radius.to_bits()
            && self.sun_intensity.to_bits() == other.sun_intensity.to_bits()
            && self.moon_angular_radius.to_bits() == other.moon_angular_radius.to_bits()
            && self.moon_intensity.to_bits() == other.moon_intensity.to_bits()
            && self.star_intensity.to_bits() == other.star_intensity.to_bits()
            && self.star_density.to_bits() == other.star_density.to_bits()
            && f3_eq(&self.ground_albedo, &other.ground_albedo)
            && self.exposure.to_bits() == other.exposure.to_bits()
            && self.palette == other.palette
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

        self.sun_position.hash(state);
        self.sun_azimuth.to_bits().hash(state);
        self.sun_angular_radius.to_bits().hash(state);
        self.sun_intensity.to_bits().hash(state);
        self.moon_angular_radius.to_bits().hash(state);
        self.moon_intensity.to_bits().hash(state);
        self.star_intensity.to_bits().hash(state);
        self.star_density.to_bits().hash(state);
        hash_f3(state, &self.ground_albedo);
        self.exposure.to_bits().hash(state);
        self.palette.hash(state);
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
    /// time-of-day sky. The sky is written directly into the cube faces and
    /// the diffuse irradiance is computed analytically (closed-form sun/moon
    /// disks + fixed quadrature), so the generated sky can be updated at
    /// runtime — see [`WorldEnvironmentDescriptor::Generated::dynamic`].
    ///
    /// The sky is driven by [`GeneratedSkyParameters`]: blue sky with a bright
    /// sun during the day, warm twilight at dawn and dusk, and a dark starry
    /// night (with a moon) after sunset.
    ///
    /// This is the **default** when no descriptor is set.
    Generated {
        cube_face_size: u32,
        sampling_type: SamplingType,
        custom_specular_mip_level_count: Option<u32>,
        /// Sky parameters. If `None`, uses [`GeneratedSkyParameters::default()`].
        parameters: Option<GeneratedSkyParameters>,
        /// When `true` the environment is expected to change frequently (e.g.
        /// an animated time-of-day). The disk cache is skipped entirely and
        /// the sky is updated **in place** on every descriptor change: the
        /// sky LoD 0 and analytic diffuse are recomputed immediately, while
        /// the specular reflection mip levels (1..N) are convolved one-per-
        /// update on a round-robin schedule (mirroring the one-shadow-per-
        /// frame light stagger). This keeps the per-frame GPU cost flat so a
        /// dynamic sky can be animated at full frame rate.
        ///
        /// See [`WorldEnvironment::update_sky_parameters`] and
        /// [`WorldEnvironment::can_update_dynamic_sky`].
        dynamic: bool,
    },
    /// Explicitly disable the environment — no skybox, no IBL lighting.
    None,
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
                    dynamic: l_dynamic,
                },
                Self::Generated {
                    cube_face_size: r_cube_face_size,
                    sampling_type: r_sampling_type,
                    custom_specular_mip_level_count: r_specular_mip_level_count,
                    parameters: r_parameters,
                    dynamic: r_dynamic,
                },
            ) => {
                l_cube_face_size == r_cube_face_size
                    && l_sampling_type == r_sampling_type
                    && l_specular_mip_level_count == r_specular_mip_level_count
                    && l_parameters == r_parameters
                    && l_dynamic == r_dynamic
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
                dynamic,
            } => {
                cube_face_size.hash(state);
                sampling_type.hash(state);
                custom_specular_mip_level_count.hash(state);
                parameters.hash(state);
                dynamic.hash(state);
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
            dynamic: false,
        };
        let b = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: None,
            dynamic: false,
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
            dynamic: false,
        };
        let mut params = GeneratedSkyParameters::default();
        params.exposure = 2.0;
        let b = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: Some(params),
            dynamic: false,
        };
        assert_ne!(a, b);

        let mut h1 = std::hash::DefaultHasher::new();
        let mut h2 = std::hash::DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn generated_descriptor_hash_differs_with_dynamic() {
        let a = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: Some(GeneratedSkyParameters::default()),
            dynamic: false,
        };
        let b = WorldEnvironmentDescriptor::Generated {
            cube_face_size: 2048,
            sampling_type: SamplingType::ImportanceSampling,
            custom_specular_mip_level_count: None,
            parameters: Some(GeneratedSkyParameters::default()),
            dynamic: true,
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
                dynamic: false,
            }
        );
    }

    /// Verify the uniform-buffer serialization matches the WGSL `SkyParams`
    /// struct layout (11 rows × 16 bytes = 176 bytes).  Must be kept in sync
    /// with `make_sky_parameters_buffer` and `generate_sky_cube.wgsl`.
    #[test]
    fn sky_params_wgsl_struct_is_176_bytes() {
        let p = GeneratedSkyParameters::default();
        let sun_dir = p.sun_direction();
        let mut b = Vec::new();
        let w = |b: &mut Vec<u8>, f: f32| b.extend_from_slice(&f.to_le_bytes());

        // Row 0: sun_direction (vec3) + _pad
        w(&mut b, sun_dir[0]);
        w(&mut b, sun_dir[1]);
        w(&mut b, sun_dir[2]);
        w(&mut b, 0.0);
        // Row 1: 4 × f32
        w(&mut b, p.sun_angular_radius);
        w(&mut b, p.sun_intensity);
        w(&mut b, p.moon_angular_radius);
        w(&mut b, p.moon_intensity);
        // Row 2: 4 × f32
        w(&mut b, p.star_intensity);
        w(&mut b, p.star_density);
        w(&mut b, p.exposure);
        w(&mut b, 0.0);
        // Row 3: ground_albedo (vec3) + _pad
        w(&mut b, p.ground_albedo[0]);
        w(&mut b, p.ground_albedo[1]);
        w(&mut b, p.ground_albedo[2]);
        w(&mut b, 0.0);
        // Rows 4-10: palette (7 × vec3 + pad each)
        for c in [
            &p.palette.day_zenith,
            &p.palette.day_horizon,
            &p.palette.night_zenith,
            &p.palette.night_horizon,
            &p.palette.twilight,
            &p.palette.sun_color,
            &p.palette.moon_color,
        ] {
            w(&mut b, c[0]);
            w(&mut b, c[1]);
            w(&mut b, c[2]);
            w(&mut b, 0.0);
        }

        assert_eq!(
            b.len(),
            176,
            "SkyParams buffer must be exactly 176 bytes (11 rows × 16 bytes)"
        );
    }

    #[test]
    fn sun_direction_from_time_of_day() {
        // 6h → dawn, sun on the horizon (elevation ~0).
        let p = GeneratedSkyParameters {
            sun_position: SunPosition::TimeOfDay { hours: 6.0 },
            ..GeneratedSkyParameters::default()
        };
        let d = p.sun_direction();
        assert!(d[1].abs() < 1e-4, "6h should be on horizon, got {}", d[1]);

        // 12h → noon, sun directly overhead.
        let p = GeneratedSkyParameters {
            sun_position: SunPosition::TimeOfDay { hours: 12.0 },
            ..GeneratedSkyParameters::default()
        };
        let d = p.sun_direction();
        assert!(
            (d[1] - 1.0).abs() < 1e-4,
            "12h should be overhead, got {}",
            d[1]
        );

        // 18h → dusk, sun on the opposite horizon.
        let p = GeneratedSkyParameters {
            sun_position: SunPosition::TimeOfDay { hours: 18.0 },
            ..GeneratedSkyParameters::default()
        };
        let d = p.sun_direction();
        assert!(d[1].abs() < 1e-4, "18h should be on horizon, got {}", d[1]);

        // 0h → midnight, sun below.
        let p = GeneratedSkyParameters {
            sun_position: SunPosition::TimeOfDay { hours: 0.0 },
            ..GeneratedSkyParameters::default()
        };
        let d = p.sun_direction();
        assert!(
            (d[1] + 1.0).abs() < 1e-4,
            "0h should be below, got {}",
            d[1]
        );
    }

    #[test]
    fn sun_direction_is_normalized() {
        let p = GeneratedSkyParameters::default();
        let d = p.sun_direction();
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-4,
            "sun direction not normalized: {len}"
        );
    }

    #[test]
    fn sun_direction_from_elevation() {
        // 90° elevation → directly overhead.
        let p = GeneratedSkyParameters {
            sun_position: SunPosition::Elevation {
                elevation_deg: 90.0,
            },
            ..GeneratedSkyParameters::default()
        };
        let d = p.sun_direction();
        assert!(
            (d[1] - 1.0).abs() < 1e-4,
            "90° should be overhead, got {}",
            d[1]
        );

        // 0° elevation → on the horizon.
        let p = GeneratedSkyParameters {
            sun_position: SunPosition::Elevation { elevation_deg: 0.0 },
            ..GeneratedSkyParameters::default()
        };
        let d = p.sun_direction();
        assert!(d[1].abs() < 1e-4, "0° should be on horizon, got {}", d[1]);

        // -90° elevation → directly below.
        let p = GeneratedSkyParameters {
            sun_position: SunPosition::Elevation {
                elevation_deg: -90.0,
            },
            ..GeneratedSkyParameters::default()
        };
        let d = p.sun_direction();
        assert!(
            (d[1] + 1.0).abs() < 1e-4,
            "-90° should be below, got {}",
            d[1]
        );
    }

    #[test]
    fn sky_palette_default_matches_constants() {
        let p = SkyPalette::default();
        assert_eq!(p.day_zenith, [0.15, 0.4, 1.0]);
        assert_eq!(p.day_horizon, [0.75, 0.85, 1.0]);
        assert_eq!(p.night_zenith, [0.005, 0.008, 0.03]);
        assert_eq!(p.night_horizon, [0.02, 0.02, 0.05]);
        assert_eq!(p.twilight, [2.0, 0.85, 0.45]);
        assert_eq!(p.sun_color, [1.0, 0.92, 0.78]);
        assert_eq!(p.moon_color, [0.72, 0.76, 0.85]);
    }

    #[test]
    fn default_star_density_in_valid_range() {
        let p = GeneratedSkyParameters::default();
        assert!((0.0..=1.0).contains(&p.star_density));
    }
}
