use std::{any::TypeId, collections::HashMap, fmt::Debug};

use crate::{Component, ComponentStore, Entity, EntityIdType, WorldComponentStorage};

#[derive(Debug)]
pub struct World {
    /// Maps an index to its current generation
    /// This is the "Source of Truth" for existence
    generations: Vec<EntityIdType>,
    /// The list of indices that are currently "free"
    free_indices: Vec<EntityIdType>,
    /// Holds all the different ComponentStore's for each Component Type.
    component_stores: HashMap<TypeId, Box<dyn WorldComponentStorage>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            component_stores: HashMap::new(),
            generations: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn is_valid(&self, entity: &Entity) -> bool {
        let idx = entity.index as usize;
        // An index is invalid if any of the following is true:
        // - Index is out of bounds -> it cannot exist yet, thus is invalid.
        // - Generation doesn't match -> existed at some point, already got replaced or is about to
        // be replaced -> thus, stale handle.
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let index = if let Some(idx) = self.free_indices.pop() {
            // If a free ID exists we take that
            idx
        } else {
            // Otherwise, create a new slot starting at generation zero
            let new_idx = self.generations.len() as EntityIdType;
            self.generations.push(0);
            new_idx
        };

        Entity::new_with_generation(index, self.generations[index])
    }

    pub fn despawn_entity(&mut self, entity: &Entity) {
        if !self.is_valid(entity) {
            return;
        }

        // First, increment the generation at entity index.
        // This will make the old entity handle "stale" and forces validation to fail.
        self.generations[entity.index] += 1;

        // Remove any components that have an attachment for the Entity
        for store in self.component_stores.values_mut() {
            let _ = store.remove_entity(entity.index);
        }

        // Mark the index as free
        self.free_indices.push(entity.index);
    }

    pub fn attach_component<C: Component>(
        &mut self,
        entity: &Entity,
        component: C,
    ) -> Result<(), ()> {
        if entity.index >= self.entity_counter {
            return Err(()); // TODO
        }

        let type_id = TypeId::of::<C>();
        let entry = self
            .component_stores
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStore::<C>::new()));
        let store = entry
            .as_any_mut()
            .downcast_mut::<ComponentStore<C>>()
            .expect("Unexpected downcasting failure at ComponentStore"); // TODO

        store.attach(entity.index, component);

        Ok(())
    }

    pub fn detach_component<C: Component>(&mut self, entity: &Entity) -> Result<(), ()> {
        if entity.index >= self.entity_counter {
            return Err(()); // TODO
        }

        let type_id = TypeId::of::<C>();

        let store = self.component_stores.get_mut(&type_id).ok_or(())?;
        store.remove_entity(entity.index);

        Ok(())
    }

    pub fn get_component_store<C: Component>(&self) -> Option<&ComponentStore<C>> {
        let type_id = TypeId::of::<C>();

        self.component_stores
            .get(&type_id)
            .and_then(|store| (**store).as_any().downcast_ref::<ComponentStore<C>>())
    }

    pub fn get_component_store_mut<C: Component>(&mut self) -> Option<&mut ComponentStore<C>> {
        let type_id = TypeId::of::<C>();

        self.component_stores
            .get_mut(&type_id)
            .and_then(|store| (**store).as_any_mut().downcast_mut::<ComponentStore<C>>())
    }
}

#[cfg(test)]
mod tests {
    use crate::World;

    #[test]
    fn spawn_entity() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        assert_eq!(0, entity.index);
        assert_eq!(0, entity.generation);
    }

    #[test]
    fn despawn_entity() {
        let mut world = World::new();

        let entity_0 = world.spawn_entity();
        assert_eq!(0, entity_0.index);
        assert_eq!(0, entity_0.generation);
        world
            .attach_component(&entity_0, String::from("First"))
            .expect("Attachment failure");
        let entity_1 = world.spawn_entity();
        assert_eq!(1, entity_1.index);
        assert_eq!(0, entity_1.generation);
        world
            .attach_component(&entity_1, String::from("Second"))
            .expect("Attachment failure");
        let entity_2 = world.spawn_entity();
        assert_eq!(2, entity_2.index);
        assert_eq!(0, entity_2.generation);
        world
            .attach_component(&entity_2, String::from("Third"))
            .expect("Attachment failure");

        world.despawn_entity(&entity_1);

        let store = world
            .get_component_store::<String>()
            .expect("Store failure");
        assert_eq!(
            store.get_component(entity_0.index),
            Some(&String::from("First"))
        );
        assert_ne!(
            store.get_component(entity_1.index),
            Some(&String::from("Second"))
        );
        assert_eq!(
            store.get_component(entity_2.index),
            Some(&String::from("Third"))
        );
    }
}
