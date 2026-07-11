//! Module trait — replaces the Element system for defining game logic.
//!
//! A `Module` is the top-level entry point for a game or application.
//! It sets up ECS entities, resources, and returns the game schedule
//! that the runtime runs each frame.

use orbital_ecs::World;
use wgpu::{Device, Queue};

use crate::Schedule;

/// Top-level game/app definition for the ECS-based engine.
///
/// Replaces the old `App` + `ElementStore` pattern. A `Module` sets up
/// all ECS state in `setup()` and returns the game schedule that the
/// runtime executes each frame alongside the core engine schedule.
pub trait Module: Send + Sync {
    /// Called once after the GPU device/queue are available.
    ///
    /// Spawn entities, insert resources, and return the game schedule.
    /// The runtime runs this schedule each frame after the core schedule.
    fn setup(ecs: &mut World, device: &Device, queue: &Queue) -> Schedule;
}
