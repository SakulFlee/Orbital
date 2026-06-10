mod id_type;
pub use id_type::*;

mod entity;
pub use entity::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub index: EntityIdType,
    pub generation: EntityIdType,
}

impl EntityId {
    pub fn new(index: EntityIdType) -> Self {
        Self {
            index,
            generation: 0,
        }
    }

    pub fn new_with_generation(index: EntityIdType, generation: EntityIdType) -> Self {
        Self { index, generation }
    }

    pub(crate) fn increment_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::Entity;

    #[test]
    fn increment_generation() {
        let mut entity = Entity::new(0);
        entity.increment_generation();

        assert_eq!(entity.index, 0);

        assert_ne!(entity.generation, 0);
        assert_eq!(entity.generation, 1);
    }

    #[test]
    fn increment_generation_loop() {
        let mut entity = Entity::new(0);

        for generation in 1..128 {
            entity.increment_generation();
            assert_eq!(entity.generation, generation);
        }
    }
}
