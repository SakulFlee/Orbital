//! Runtime rendering controls shared by the app runtime and the debug UI.

use orbital_renderer::{AntiAliasing, SmaaQuality};

use crate::render_scale::MAX_RENDER_SCALE;

/// Renderer settings that can be changed while the app is running.
///
/// [`ModuleRuntime`](crate::ModuleRuntime) applies this resource before every
/// redraw. Insert it during module setup to override the startup
/// [`AppSettings`](crate::AppSettings), or mutate it from a system or panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderSettings {
    /// Scale factor for the internal 3D render resolution.
    ///
    /// `1.0` renders at the surface resolution; `0.5` renders at half width
    /// and height (a quarter of the pixels) and upscales during tonemapping.
    /// 2D, UI, and text overlays always stay at native surface resolution.
    pub render_scale: f32,
    /// Automatically adjust [`Self::render_scale`] to hold
    /// [`Self::target_fps`].
    ///
    /// The runtime changes the scale at most once per one-second timing
    /// window. Manual scale changes disable this flag so an explicit choice is
    /// not immediately overwritten.
    pub adaptive_render_scale: bool,
    /// Desired frame rate for adaptive render scaling.
    ///
    /// Capped by the display refresh rate when it is known and by
    /// [`AppSettings::fps_limit`](crate::AppSettings::fps_limit) when a frame
    /// cap is configured.
    pub target_fps: u32,
    /// Lower bound for adaptive render scaling.
    pub min_render_scale: f32,
    /// Upper bound for adaptive render scaling.
    pub max_render_scale: f32,
    /// 3D scene anti-aliasing mode.
    pub anti_aliasing: AntiAliasing,
    /// Quality preset used when `anti_aliasing` is [`AntiAliasing::Smaa`].
    pub smaa_quality: SmaaQuality,
    /// Exposure multiplier applied before the tonemap curve.
    pub exposure: f32,
    /// Whether HDR glare is sampled during tonemapping.
    pub bloom_enabled: bool,
    /// HDR brightness above which glare starts to contribute.
    pub bloom_threshold: f32,
    /// Strength of the sampled glare.
    pub bloom_intensity: f32,
    /// Whether positional lights contribute screen-space glare.
    pub light_glare: bool,
    /// Contrast applied after the tonemap curve.
    pub contrast: f32,
    /// Use deferred lighting for opaque PBR models.
    pub deferred_enabled: bool,
    /// Whether positional light glare gets diffraction stars.
    pub glare_stars: bool,
    /// Number of points on the star.
    pub glare_star_arms: u32,
    /// Star intensity near the light core.
    pub glare_star_inner: f32,
    /// Star intensity toward the arm tips.
    pub glare_star_outer: f32,
    /// Arm length as a multiple of the glare radius.
    pub glare_star_length: f32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            render_scale: 1.0,
            adaptive_render_scale: cfg!(any(target_os = "android", target_os = "ios")),
            target_fps: 60,
            min_render_scale: 0.25,
            max_render_scale: MAX_RENDER_SCALE,
            anti_aliasing: AntiAliasing::Fxaa,
            smaa_quality: SmaaQuality::default(),
            exposure: 1.0,
            bloom_enabled: false,
            bloom_threshold: 4.0,
            bloom_intensity: 0.65,
            light_glare: false,
            contrast: 1.0,
            deferred_enabled: true,
            glare_stars: true,
            glare_star_arms: 8,
            glare_star_inner: 0.05,
            glare_star_outer: 0.0,
            glare_star_length: 10.0,
        }
    }
}
