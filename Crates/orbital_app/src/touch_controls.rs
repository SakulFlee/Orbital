//! Configurable tuning for the engine's default touch controls.
//!
//! Stored in a process-global (rather than an ECS resource) so the touch
//! camera controller — whose `IntoSystem` signature is limited to
//! `Res + Res + &mut + &mut` — and the on-screen joystick overlay can both
//! read the same values without changing their system signatures.

use std::sync::RwLock;

/// Tuning parameters for the default touch control scheme.
///
/// Set them once at startup via [`set_touch_controls`], or mutate the global
/// in place via [`touch_controls`] (e.g. from a settings screen). The
/// defaults match the pre-configuration constants.
#[derive(Debug, Clone)]
pub struct TouchControlsConfig {
    /// Camera movement speed in units per second (joystick at full deflection).
    pub move_speed: f32,
    /// Drag-to-look sensitivity in radians per screen pixel.
    pub look_sensitivity: f32,
    /// Maximum pitch (radians) for touch look, preventing the up-vector flip.
    pub pitch_limit: f32,
    /// Virtual joystick radius in window pixels (full deflection).
    pub joystick_radius: f64,
    /// Joystick dead-zone radius in window pixels.
    pub joystick_deadzone: f64,
    /// Whether to draw idle "move"/"look" hint indicators on touch devices.
    pub show_idle_hints: bool,
}

impl Default for TouchControlsConfig {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            look_sensitivity: 0.005,
            pitch_limit: 1.45,
            joystick_radius: 70.0,
            joystick_deadzone: 10.0,
            show_idle_hints: true,
        }
    }
}

/// Process-global touch control configuration.
///
/// Initialised with the same values as [`TouchControlsConfig::default`]
/// (a `const` initializer is required for the static).
static TOUCH_CONTROLS: RwLock<TouchControlsConfig> = RwLock::new(TouchControlsConfig {
    move_speed: 5.0,
    look_sensitivity: 0.005,
    pitch_limit: 1.45,
    joystick_radius: 70.0,
    joystick_deadzone: 10.0,
    show_idle_hints: true,
});

/// Read the current touch control configuration.
pub fn touch_controls() -> TouchControlsConfig {
    TOUCH_CONTROLS
        .read()
        .expect("TouchControlsConfig lock poisoned")
        .clone()
}

/// Replace the touch control configuration (e.g. from a settings screen).
pub fn set_touch_controls(config: TouchControlsConfig) {
    *TOUCH_CONTROLS
        .write()
        .expect("TouchControlsConfig lock poisoned") = config;
}
