//! # Orbital Engine
//!
//! A multi-platform 3D rendering engine built in Rust using wgpu as the graphics backend.

pub use orbital_app as app;
pub use orbital_ecs as ecs;
pub use orbital_ecs_bridge as ecs_bridge;
pub use orbital_importer_gltf as importer;
pub use orbital_renderer as renderer;
pub use orbital_resources as resources;
pub use orbital_shader_preprocessor as shader_preprocessor;
pub use orbital_debug_render as debug_render;

#[cfg(test)]
pub mod wgpu_test_adapter;

pub use orbital_core::{cache, logging, macros, mip_level, or, quaternion};
pub use orbital_core::{make_android_main, make_desktop_main};

// Re-exports
pub use cgmath;
#[cfg(feature = "gamepad_input")]
pub use gilrs;
pub use wgpu;
pub use winit;
