#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A unique identifier for an entity in the ECS world.
///
/// The entity ID consists of an index and a generation counter. The index
/// refers to the entity's position in the entity storage, while the generation
/// counter is used to detect stale references (e.g., after an entity has been
/// despawned and a new entity has been created at the same index).
pub struct Entity {
    /// The index of the entity in the storage.
    pub index: usize,
    /// The generation counter used to prevent use-after-free.
    pub generation: usize,
}

impl Entity {
    /// Creates a new Entity with the given index and generation.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the entity in the storage.
    /// * `generation` - The generation counter for the entity.
    ///
    /// # Returns
    ///
    /// A new Entity instance.
    pub fn new(index: usize, generation: usize) -> Self {
        Self { index, generation }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_creation() {
        let e = Entity::new(0, 0);
        assert_eq!(e.index, 0);
        assert_eq!(e.generation, 0);
    }

    #[test]
    fn entity_eq() {
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(0, 0);
        let e3 = Entity::new(0, 1);
        let e4 = Entity::new(1, 0);

        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
        assert_ne!(e1, e4);
    }

    #[test]
    fn entity_hash() {
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(0, 0);
        let e3 = Entity::new(0, 1);

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut s1 = DefaultHasher::new();
        let mut s2 = DefaultHasher::new();
        let mut s3 = DefaultHasher::new();

        e1.hash(&mut s1);
        e2.hash(&mut s2);
        e3.hash(&mut s3);

        assert_eq!(s1.finish(), s2.finish());
        assert_ne!(s1.finish(), s3.finish());
    }
}
