//! # Resources Module (Re-export Shim)
//!
//! This crate re-exports all resource types from their new fine-grained
//! crates. It exists purely for backward compatibility — new code should
//! import directly from the specific crate.

// Flat re-exports for backward compatibility
pub use orbital_camera::*;
pub use orbital_cull::*;
pub use orbital_ibl_brdf::*;
pub use orbital_instance::*;
pub use orbital_light::*;
pub use orbital_material_shader::*;
pub use orbital_math::{Mode, Transform, ortho_wgpu, perspective_wgpu};
pub use orbital_mesh::*;
pub use orbital_model::*;
pub use orbital_shader_core::*;
pub use orbital_shadow::*;
pub use orbital_texture::*;
pub use orbital_vertex::*;
pub use orbital_world_environment::*;

// Re-export PBR material shader types
pub use orbital_shader_pbr::*;
