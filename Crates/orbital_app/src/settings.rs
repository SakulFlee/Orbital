use std::time::Duration;

use orbital_renderer::{AntiAliasing, SmaaQuality};
use winit::dpi::{PhysicalSize, Size};

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub name: String,
    pub size: Size,
    pub vsync_enabled: bool,
    /// 3D scene anti-aliasing. 2D/UI overlays are always rendered after the
    /// post chain and are not affected.
    ///
    /// This is the startup value; insert or change [`crate::RenderSettings`]
    /// to change anti-aliasing at runtime.
    pub anti_aliasing: AntiAliasing,
    /// Quality preset used when `anti_aliasing` is [`AntiAliasing::Smaa`].
    pub smaa_quality: SmaaQuality,
    /// Exposure multiplier applied before the tonemap curve.
    pub exposure: f32,
    /// Whether HDR glare is sampled during tonemapping.
    pub bloom_enabled: bool,
    /// HDR brightness above which scene glare starts to contribute.
    pub bloom_threshold: f32,
    /// Strength of the sampled glare.
    pub bloom_intensity: f32,
    /// Number of consecutive back-presses (within `back_exit_window`) that
    /// quit the app. `0` disables the behavior entirely, leaving back input
    /// fully up to the app.
    pub back_presses_to_exit: u8,
    /// Time window in which consecutive back-presses must arrive to trigger
    /// an exit. Only used when `back_presses_to_exit > 0`.
    pub back_exit_window: Duration,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "Orbital App".into(),
            size: PhysicalSize::new(1280, 720).into(),
            vsync_enabled: true,
            anti_aliasing: AntiAliasing::Fxaa,
            smaa_quality: SmaaQuality::default(),
            exposure: 1.0,
            bloom_enabled: false,
            bloom_threshold: 4.0,
            bloom_intensity: 0.65,
            back_presses_to_exit: 0,
            back_exit_window: Duration::from_secs(2),
        }
    }
}
