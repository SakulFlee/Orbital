//! Module trait — replaces the Element system for defining game logic.
//!
//! A `Module` is a plugin-like entry point for game or engine functionality.
//! Multiple modules can coexist — each contributes systems to the game schedule.
//! The runtime merges all module systems into a single game schedule.

use orbital_ecs::World;
use orbital_ecs::System;
use wgpu::{Device, Queue};

/// Plugin-like module for the ECS-based engine.
///
/// Replaces the old `App` + `ElementStore` pattern. A `Module` sets up
/// ECS entities and resources in `setup()`, then returns systems that
/// run each frame. Multiple modules can be registered — their systems
/// are all merged into the game schedule.
pub trait Module: Send + Sync {
    /// Called once after the GPU device/queue are available.
    ///
    /// Spawn entities, insert resources, and return the systems this
    /// module contributes to the game schedule. The runtime merges
    /// systems from all modules into a single schedule.
    fn setup(&self, ecs: &mut World, device: &Device, queue: &Queue) -> Vec<Box<dyn System>>;
}
