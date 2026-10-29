//! Runtime rendering controls shared by the app runtime and the debug UI.

use orbital_renderer::{AntiAliasing, SmaaQuality};

/// Renderer settings that can be changed while the app is running.
///
/// [`ModuleRuntime`](crate::ModuleRuntime) applies this resource before every
/// redraw. Insert it during module setup to override the startup
/// [`AppSettings`](crate::AppSettings), or mutate it from a system or panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderSettings {
    /// 3D scene anti-aliasing mode.
    pub anti_aliasing: AntiAliasing,
    /// Quality preset used when `anti_aliasing` is [`AntiAliasing::Smaa`].
    pub smaa_quality: SmaaQuality,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            anti_aliasing: AntiAliasing::Fxaa,
            smaa_quality: SmaaQuality::default(),
        }
    }
}
