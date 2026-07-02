use std::{any::Any, fmt::Debug};

use crate::ComponentStore;

pub trait WorldComponentStorage: Debug + Send + Sync {
    fn as_any<'a>(&'a self) -> &'a dyn Any;

    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn Any;

    fn remove_entity(&mut self, entity_id: usize) -> bool;
}

impl<T: Any + Debug + Send + Sync> WorldComponentStorage for ComponentStore<T> {
    fn as_any<'a>(&'a self) -> &'a dyn Any {
        self
    }

    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn Any {
        self
    }

    fn remove_entity(&mut self, entity_id: usize) -> bool {
        self.detach(entity_id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};

    use crate::{ComponentStore, WorldComponentStorage};

    #[test]
    fn ensure_upcast() {
        let store = ComponentStore::<usize>::new();
        let world_store: Box<dyn WorldComponentStorage> = Box::new(store);
        assert_eq!(
            TypeId::of::<Box<dyn WorldComponentStorage>>(),
            world_store.type_id()
        );
    }

    #[test]
    fn ensure_downcast() {
        let store = ComponentStore::<usize>::new();
        let world_store: Box<dyn WorldComponentStorage> = Box::new(store);

        if let Some(downcasted) = world_store.as_any().downcast_ref::<ComponentStore<usize>>() {
            assert_eq!(TypeId::of::<ComponentStore<usize>>(), downcasted.type_id(),);
        } else {
            panic!("Downcasting failed!");
        }
    }

    #[test]
    fn ensure_remove_entity() {
        let mut store = ComponentStore::<usize>::new();
        store.attach(0, 111);
        store.attach(1, 222);
        store.attach(2, 333);

        let mut world_store: Box<dyn WorldComponentStorage> = Box::new(store);

        // Remove entity, verify result
        let result = world_store.remove_entity(1);
        assert!(result);

        if let Some(casted_store) = world_store.as_any().downcast_ref::<ComponentStore<usize>>() {
            // Test deletion to have happened
            assert_ne!(casted_store.get_component(1), Some(&222));
            assert_eq!(casted_store.get_component(1), None);

            // Validate other data is untouched
            assert_eq!(casted_store.get_component(0), Some(&111));
            assert_eq!(casted_store.get_component(2), Some(&333));
        }
    }
}
