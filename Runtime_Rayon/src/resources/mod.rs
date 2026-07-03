//! # Resources Module
//!
//! The resources module contains all the core resource types used by the Orbital engine.
//! These resources represent the fundamental building blocks for 3D scenes, including
//! models, cameras, textures, materials, and lighting.
//!
//! ## Key Resource Types
//!
//! - **Model**: Represents 3D models with meshes, materials, and instances
//! - **Camera**: Manages view and projection matrices for rendering
//! - **Texture**: Handles image data for materials and environment mapping
//! - **Light**: Represents different types of lighting in the scene
//! - **Shader**: Manages shader programs and pipeline creation
//! - **WorldEnvironment**: Handles environment mapping and IBL (Image-Based Lighting)
//!
//! ## Resource Lifecycle
//!
//! Resources follow a specific lifecycle involving creation, realization, caching,
//! and cleanup. The engine manages resource lifecycles automatically through
//! the various stores in the world module.

pub mod bounding_box;
pub mod buffer;
pub mod camera;
pub mod debug_material_shader;
pub mod ibl_brdf;
pub mod instance;
pub mod light;
pub mod material_shader;
pub mod mesh;
pub mod model;
pub mod pbr_material_shader;
pub mod shader;
pub mod texture;
pub mod transform;
pub mod vertex;
pub mod world_environment;

pub use bounding_box::*;
pub use buffer::*;
pub use camera::*;
pub use debug_material_shader::*;
pub use ibl_brdf::*;
pub use instance::*;
pub use light::*;
pub use material_shader::*;
pub use mesh::*;
pub use model::*;
pub use pbr_material_shader::*;
pub use shader::*;
pub use texture::*;
pub use transform::*;
pub use vertex::*;
pub use world_environment::*;
