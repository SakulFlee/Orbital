mod settings;
pub use settings::*;

mod core_schedule;
pub use core_schedule::*;

pub use orbital_ecs::Schedule;

mod timer;
pub use timer::*;

mod context;
pub use context::*;

mod state;
pub use state::*;

pub mod app;
pub use app::App;

pub mod module;
pub use module::Module;

pub mod module_runtime;
pub use module_runtime::ModuleRuntime;

pub mod render_overlay;
pub use render_overlay::{RenderOverlay, RenderOverlayContext, RenderOverlayResource};

mod touch_controls;
pub use touch_controls::{TouchControlsConfig, set_touch_controls, touch_controls};

/// Key used by the freeze-frustum toggle (default F4).
///
/// Set by [`DebugModule`](orbital_debug_render::DebugModule) during setup.
/// Read in [`ModuleRuntime::redraw`].
#[derive(Debug, Clone)]
pub struct FreezeKeyConfig(pub winit::keyboard::KeyCode);

pub mod systems;
pub use systems::{
    realize_cameras, realize_environment, realize_lights, realize_models, sys_camera_controller,
    sys_frustum_cull, sys_touch_camera_controller,
};

pub use orbital_input as input;
pub use orbital_input::*;
