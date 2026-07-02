//! # Orbital Engine
//!
//! A multi-platform 3D rendering engine built in Rust using wgpu as the graphics backend.
//! The engine provides a flexible framework for creating 3D applications and games with
//! support for modern rendering techniques including PBR (Physically Based Rendering),
//! IBL (Image Based Lighting), and GLTF asset import.
//!
//! ## Architecture
//!
//! The engine is organized into several key modules:
//!
//! - [**app**](app): Application lifecycle management and cross-platform entry points
//! - [**world**](world): World state management and resource stores
//! - [**element**](element): Game objects and entity-component-like system with messaging
//! - [**resources**](resources): Core resource types (models, cameras, textures, materials)
//! - [**renderer**](renderer): Rendering pipeline and draw commands
//! - [**importer**](importer): Asset import functionality, primarily GLTF
//! - [**camera_controller**](camera_controller): Camera control system with various movement types
//!
//! ## Key Concepts
//!
//! - [**Elements**](element): The primary way to add interactive objects to your world.
//!   Elements are the core game objects that exist in the world and handle their own behavior.
//!   They communicate with each other and the world through a message-passing system rather than shared memory.
//!   Each element can register itself with the world to define what resources it needs (models, cameras, etc.),
//!   and can respond to various events during the application lifecycle such as updates and messages from other elements.
//! - [**World**](world): Contains all resources and manages their lifecycle
//! - [**App**](app): The main application trait that handles platform-specific events
//! - [**Messaging**](element): Elements communicate through a message-passing system rather than shared memory
//!
//! ## Getting Started
//!
//! Implement the [app::App] trait to create your application, then register [element::Element]s
//! with the world to create your scene. The engine handles rendering, input, and resource
//! management automatically.

pub mod app;
pub mod cache;
pub mod camera_controller;
pub mod element;
pub mod importer;
pub mod logging;
pub mod macros;
pub mod mip_level;
pub mod or;
pub mod quaternion;
pub mod renderer;
pub mod resources;
pub mod shader_preprocessor;
pub mod world;

#[cfg(test)]
pub mod wgpu_test_adapter;

// Re-exports
pub use async_std;
pub use async_trait;
pub use cgmath;
pub use futures;
#[cfg(feature = "gamepad_input")]
pub use gilrs;
pub use wgpu;
pub use winit;
