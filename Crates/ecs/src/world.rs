use std::{any::TypeId, collections::HashMap, fmt::Debug};

use crate::{Component, ComponentStore, Entity, EntityIdType, WorldComponentStorage};

#[derive(Debug)]
pub struct World {
    /// Indexer over Entity IDs.
    /// Indicates the next free Entity ID.
    ///
    /// > Must be incremented after use!
    entity_counter: EntityIdType,
    /// Holds any "freed" (despawned) Entity IDs.
    /// When spawning a new Entity ID, these freed IDs are prevert.
    ///
    /// > The generation counter has to be incremented of each freed Entity ID before use!
    entities_freed: Vec<Entity>,
    /// When Entities are freshly spawned, they aren't attached to any Components yet.
    /// To not loose track of these reserved IDs, and additional data like generations, we
    /// temporarily park them here _until_ a Component gets attached to the Entity.
    entities_without_components: Vec<Entity>,
    /// Holds all the different ComponentStore's for each Component Type.
    component_stores: HashMap<TypeId, Box<dyn WorldComponentStorage>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
            entities_freed: Vec::new(),
            entities_without_components: Vec::new(),
            component_stores: HashMap::new(),
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        // If a freed Entity ID is available, we take it from the list and reuse it.
        // Otherwise, we create a new one and increment the counter.
        let mut entity = self.entities_freed.pop().unwrap_or_else(|| {
            let next_entity_id = self.entity_counter;
            self.entity_counter = self.entity_counter.wrapping_add(1);

            Entity::new(next_entity_id)
        });

        // Always increment the generation.
        // This means a generation will always start at 1.
        entity.increment_generation();

        // Temporarily park the Entity ID away _until_ a Component is attached.
        self.entities_without_components.push(entity);

        entity
    }

    pub fn despawn_entity(&mut self, entity: &Entity) {
        if entity.index > self.entity_counter {
            return; // Entity cannot exist (yet)
        }

        // Store current length of Vector -> retain -> check if something changed
        let len = self.entities_without_components.len();
        self.entities_without_components
            .retain(|x| x.index != entity.index);
        if self.entities_without_components.len() != len {
            self.entities_freed.push(*entity);
            return;
            // Early return as we found an entity without any components, thus it cannot be
            // in any stores.
        }

        // Remove any components that have an attachment for the Entity
        if self
            .component_stores
            .iter_mut()
            .any(|(_, x)| x.remove_entity(entity.index))
        {
            self.entities_freed.push(*entity);
        }
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
        assert_eq!(1, entity_0.generation);
        world
            .attach_component(&entity_0, String::from("First"))
            .expect("Attachment failure");
        let entity_1 = world.spawn_entity();
        assert_eq!(1, entity_1.index);
        assert_eq!(1, entity_1.generation);
        world
            .attach_component(&entity_1, String::from("Second"))
            .expect("Attachment failure");
        let entity_2 = world.spawn_entity();
        assert_eq!(2, entity_2.index);
        assert_eq!(1, entity_2.generation);
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
