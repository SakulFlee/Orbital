use std::time::Duration;

use winit::dpi::{PhysicalSize, Size};

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub name: String,
    pub size: Size,
    pub vsync_enabled: bool,
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
            back_presses_to_exit: 0,
            back_exit_window: Duration::from_secs(2),
        }
    }
}
