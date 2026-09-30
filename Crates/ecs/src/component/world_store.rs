use std::{any::Any, fmt::Debug};

use crate::ComponentStore;

/// A type-erased [`ComponentStore`] owned by a [`World`](crate::World).
///
/// The [`World`](crate::World) keeps one of these per registered component
/// type, which makes this trait the single place where introspection support
/// lives: it can report which component type it stores, which entities hold it
/// and what those values are, all without the caller naming `T`.
pub trait WorldComponentStorage: Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn remove_entity(&mut self, entity_id: usize) -> bool;

    /// The [`TypeId`](std::any::TypeId) of the component type this store holds.
    fn component_type_id(&self) -> std::any::TypeId;

    /// The fully qualified Rust type name of the stored component type.
    ///
    /// For example `"my_game::components::Position"`.
    fn component_type_name(&self) -> &'static str;

    /// The indices of every entity that currently holds this component.
    ///
    /// This is the store's `dense` set, so it contains no stale entries. The
    /// order is the order components were attached, except that detaching
    /// swap-removes and so may move the last entry into the gap.
    fn entity_indices(&self) -> &[usize];

    /// The stored value for `entity_id`, as a type-erased [`Debug`].
    ///
    /// Returns `None` when the entity does not hold this component.
    fn component_debug(&self, entity_id: usize) -> Option<&dyn Debug>;

    /// The stored value for `entity_id`, as a type-erased [`Any`].
    ///
    /// Returns `None` when the entity does not hold this component. Callers can
    /// [`downcast_ref`](Any::downcast_ref) to a concrete type when they know it.
    fn component_any(&self, entity_id: usize) -> Option<&dyn Any>;

    /// Replaces the stored value for `entity_id` with `component`.
    ///
    /// `component` is downcast to the stored type; if it does not match, the
    /// value is left untouched and `false` is returned.
    ///
    /// Returns `false` when the entity does not already hold this component —
    /// use [`World::attach_component`](crate::World::attach_component) to add
    /// one.
    fn set_component_any(&mut self, entity_id: usize, component: Box<dyn Any>) -> bool;

    /// Replaces the stored value for `entity_id` with `component`, attaching it
    /// first if the entity does not already hold this component.
    ///
    /// Returns `false` if `component` is not of the stored type.
    fn set_or_insert_component_any(&mut self, entity_id: usize, component: Box<dyn Any>) -> bool;
}

impl<T: Any + Debug + Send + Sync> WorldComponentStorage for ComponentStore<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_entity(&mut self, entity_id: usize) -> bool {
        self.detach(entity_id).is_some()
    }

    fn component_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }

    fn component_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn entity_indices(&self) -> &[usize] {
        &self.dense
    }

    fn component_debug(&self, entity_id: usize) -> Option<&dyn Debug> {
        self.get_component(entity_id).map(|c| c as &dyn Debug)
    }

    fn component_any(&self, entity_id: usize) -> Option<&dyn Any> {
        self.get_component(entity_id).map(|c| c as &dyn Any)
    }

    fn set_component_any(&mut self, entity_id: usize, component: Box<dyn Any>) -> bool {
        let Ok(component) = component.downcast::<T>() else {
            return false;
        };

        let Some(component_index) = self.sparse.get(entity_id).copied().flatten() else {
            return false;
        };

        self.components[component_index] = *component;
        true
    }

    fn set_or_insert_component_any(&mut self, entity_id: usize, component: Box<dyn Any>) -> bool {
        let Ok(component) = component.downcast::<T>() else {
            return false;
        };

        match self.sparse.get(entity_id).copied().flatten() {
            Some(component_index) => self.components[component_index] = *component,
            None => self.attach(entity_id, *component),
        }

        true
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
