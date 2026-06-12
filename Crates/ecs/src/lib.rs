/// The Entity Component System (ECS) library.
///
/// This library provides an archetype-based ECS implementation designed for
/// game development and simulation. It includes:
///
/// * Entities with generation counters to prevent use-after-free
/// * Type-safe component storage
/// * Archetype-based entity grouping for cache efficiency
/// * World management for entity and component lifecycle
///
/// # Example
///
/// ```
/// use ecs::{World, Component};
///
/// #[derive(Debug)]
/// struct Position {
///     x: f32,
///     y: f32,
/// }
///
/// fn main() {
///     let mut world = World::new();
///     let entity = world.spawn_entity();
///     assert!(world.is_valid(&entity));
///     world.despawn_entity(&entity);
///     assert!(!world.is_valid(&entity));
/// }
/// ```
pub mod entity;
pub use entity::*;

pub mod component;
pub use component::*;

pub mod world;
pub use world::World;

pub mod archetype;
pub use archetype::{Archetype, ArchetypeManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exports() {
        // This test ensures that all the public items can be imported.
        let _ = Entity::new(0, 0);
        let _ = World::new();
    }
}
