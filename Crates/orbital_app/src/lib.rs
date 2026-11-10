mod render_settings;
pub use render_settings::*;

mod settings;
pub use settings::*;

#[allow(dead_code)]
pub mod android_performance;

mod core_schedule;
pub use core_schedule::*;

pub use orbital_ecs::Schedule;

mod timer;
pub use timer::*;

mod frame_limiter;
pub use frame_limiter::FrameLimiter;

mod render_scale;
pub use render_scale::{
    AdaptiveRenderScale, MAX_RENDER_SCALE, MIN_RENDER_SCALE, effective_target_fps,
    scaled_resolution,
};

mod performance_csv;
pub use performance_csv::{PERFORMANCE_CSV_PATH, PERFORMANCE_METADATA_PATH};

mod performance_metadata;

mod context;
pub use context::*;

mod state;
pub use state::*;

pub mod app;
pub use app::App;

mod camera_commands;
pub use camera_commands::{CameraCommand, apply_camera_commands};

pub mod module;
pub use module::Module;

pub mod module_runtime;
pub use module_runtime::ModuleRuntime;

pub mod render_layer;
pub use render_layer::{LayerConfig, RenderLayer};

pub mod render_overlay;
pub use render_overlay::{
    LayerRenderer, OverlayRenderTimings, OverlaySubmission, RenderOverlayContext,
    RenderOverlayResource,
};

mod touch_controls;
pub use touch_controls::{TouchControlsConfig, set_touch_controls, touch_controls};

#[cfg(all(feature = "gamepad_input_poll", target_os = "ios"))]
mod ios_gamepad;

/// Key used by the freeze-frustum toggle (default F4).
///
/// Set by [`DebugModule`](orbital_debug_render::DebugModule) during setup.
/// Read in [`ModuleRuntime::redraw`].
#[derive(Debug, Clone)]
pub struct FreezeKeyConfig(pub winit::keyboard::KeyCode);

pub mod systems;
pub use systems::{
    CameraControlSettings, CursorToggle, OrthographicForwardMode, realize_cameras,
    realize_environment, realize_lights, realize_models, sys_camera_controller, sys_frustum_cull,
    sys_ortho_dolly, sys_ortho_pan, sys_ortho_zoom, sys_touch_camera_controller,
};

pub use orbital_input as input;
pub use orbital_input::*;
