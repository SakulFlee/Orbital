//! # Orbital ECS Bridge
//!
//! Bridge types that connect the ECS library (`orbital_ecs`) with engine-specific
//! state — frame timing, input, window, and GPU resources. These types are designed
//! to be inserted into `orbital_ecs::World` as **resources** via
//! `world.insert_resource(...)`, making them accessible to ECS systems through
//! `Res<T>` / `ResMut<T>` parameters.
//!
//! This crate exists to keep `orbital_ecs` engine-agnostic while providing
//! the glue layer that Orbital's runtime, renderer, and game code depend on.

pub mod resources;
pub use resources::*;
