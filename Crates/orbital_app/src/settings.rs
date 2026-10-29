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
    pub anti_aliasing: AntiAliasing,
    /// Quality preset used when `anti_aliasing` is [`AntiAliasing::Smaa`].
    pub smaa_quality: SmaaQuality,
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
            back_presses_to_exit: 0,
            back_exit_window: Duration::from_secs(2),
        }
    }
}
