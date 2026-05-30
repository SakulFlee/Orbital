use std::{any::Any, fmt::Debug};

use crate::{ComponentStore, EntityIdType};

pub trait WorldComponentStorage: Debug {
    fn as_any<'a>(&'a self) -> &'a dyn Any;

    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn Any;

    fn remove_entity(&mut self, entity_id: EntityIdType) -> bool;
}

impl<T: Any + Debug> WorldComponentStorage for ComponentStore<T> {
    fn as_any<'a>(&'a self) -> &'a dyn Any {
        self
    }

    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn Any {
        self
    }

    fn remove_entity(&mut self, entity_id: EntityIdType) -> bool {
        self.detach(entity_id).is_some()
    }
}
