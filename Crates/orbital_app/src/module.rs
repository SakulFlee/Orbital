//! Module trait — replaces the Element system for defining game logic.
//!
//! A `Module` is a plugin-like entry point for game or engine functionality.
//! Multiple modules can coexist — each contributes systems to the game schedule.
//! The runtime merges all module systems into a single game schedule.

use orbital_ecs::System;
use orbital_ecs::World;
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

    /// Called when the app is paused/backgrounded.
    ///
    /// The OS may kill the process at any point (OOM, aggressive battery
    /// management), so persist any state here that should survive a restart
    /// (e.g. via [`orbital_core::file_manager::FileManager`]).
    fn save_state(&self, _ecs: &mut World) {}

    /// Called once, right after [`Module::setup`], on a fresh process start.
    ///
    /// Restore state previously persisted in [`Module::save_state`]. Not called
    /// on a true in-memory resume (activity merely paused).
    fn restore_state(&self, _ecs: &mut World) {}

    /// Called when the app exits cleanly (triple-back / close request).
    ///
    /// Delete any saved state so the next launch starts fresh.
    fn clear_state(&self, _ecs: &mut World) {}
}
