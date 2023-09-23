use std::mem::discriminant;

use super::BoxedEntity;

pub enum EntityAction {
    Spawn(Vec<BoxedEntity>),
    Remove(Vec<String>),
    Keep,
}

impl PartialEq for EntityAction {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}
