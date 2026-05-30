mod id_type;
pub use id_type::*;

mod entity;
pub use entity::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub index: EntityIdType,
    pub generation: u32,
}

impl EntityId {
    pub fn new(index: EntityIdType) -> Self {
        Self {
            index,
            generation: 0,
        }
    }

    pub(crate) fn increment_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}
