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

pub mod buffer;
pub mod camera;
pub mod cull;
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

pub use buffer::*;
pub use camera::*;
pub use cull::*;
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

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Device, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension,
};

pub fn make_world_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("World BindGroup Layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::all(),
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::all(),
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::Cube,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::all(),
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::all(),
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::Cube,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 5,
                visibility: ShaderStages::all(),
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 6,
                visibility: ShaderStages::all(),
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 7,
                visibility: ShaderStages::all(),
                ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    })
}
