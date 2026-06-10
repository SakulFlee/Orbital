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

    /// An index is invalid if any of the following is true:
    /// - Index is out of bounds -> it cannot exist yet, thus is invalid.
    /// - Generation doesn't match -> existed at some point, already got replaced or is about to
    /// be replaced -> thus, stale handle.
    pub fn is_valid(&self, entity: &Entity) -> bool {
        let idx = entity.index as usize;
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

        // First, we increment the generation.
        // This will automatically invalidate all existing Entity handles pointing to this index.
        self.generations[entity.index] = self.generations[entity.index].wrapping_add(1);

        // Then, remove the entity from all component stores.
        for store in self.component_stores.values_mut() {
            store.remove_entity(entity.index);
        }

        // Lastly, Add the index to the free pool for reuse.
        self.free_indices.push(entity.index);
    }

    pub fn attach_component<C: Component>(
        &mut self,
        entity: &Entity,
        component: C,
    ) -> Result<(), ()> {
        if !self.is_valid(entity) {
            return Err(());
        }

        let type_id = TypeId::of::<C>();
        let entry = self
            .component_stores
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStore::<C>::new()));

        let store = entry
            .as_any_mut()
            .downcast_mut::<ComponentStore<C>>()
            .expect("Unexpected downcasting failure at ComponentStore");

        store.attach(entity.index, component);

        Ok(())
    }

    pub fn detach_component<C: Component>(&mut self, entity: &Entity) -> Result<(), ()> {
        if !self.is_valid(entity) {
            return Err(());
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
    use super::*;

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

    #[test]
    fn test_index_reuse_and_generation_increment() {
        let mut world = World::new();

        // Spawn first entity
        let e1 = world.spawn_entity();
        let idx1 = e1.index;
        let gen1 = e1.generation;
        assert_eq!(idx1, 0);
        assert_eq!(gen1, 0);

        // Despawn it
        world.despawn_entity(&e1);

        // Spawn second entity - should reuse index 0 but have generation 1
        let e2 = world.spawn_entity();
        assert_eq!(e2.index, idx1, "Should reuse the freed index");
        assert_ne!(e2.generation, gen1, "Generation should have incremented");
        assert_eq!(e2.generation, 1);
    }

    #[test]
    fn test_stale_handle_invalidation() {
        let mut world = World::new();

        let e1 = world.spawn_entity();
        world.despawn_entity(&e1);

        // e1 is now a stale handle
        assert!(
            !world.is_valid(&e1),
            "Handle should be invalid after despawn"
        );

        // Attempting to attach a component to a stale handle should fail
        let result = world.attach_component(&e1, String::from("Ghost"));
        assert!(
            result.is_err(),
            "Should not allow attaching components to stale handles"
        );
    }

    #[test]
    fn test_complex_reuse_pattern() {
        let mut world = World::new();

        // Spawn 3 entities
        let e0 = world.spawn_entity(); // idx 0, gen 0
        let e1 = world.spawn_entity(); // idx 1, gen 0
        let e2 = world.spawn_entity(); // idx 2, gen 0

        // Despawn middle one
        world.despawn_entity(&e1);

        // Spawn a new one - should take index 1
        let e1_new = world.spawn_entity();
        assert_eq!(e1_new.index, e1.index);
        assert_eq!(e1_new.generation, 1);

        // Verify e0 and e2 are still valid
        assert!(world.is_valid(&e0));
        assert!(world.is_valid(&e2));
        assert!(world.is_valid(&e1_new));

        // Verify e1 is still invalid
        assert!(!world.is_valid(&e1));
    }

    #[test]
    fn test_out_of_bounds_validation() {
        let world = World::new();

        // Manually construct an entity with an out-of-bounds index
        let fake_entity = Entity::new_with_generation(999, 0);

        assert!(
            !world.is_valid(&fake_entity),
            "Out of bounds index should be invalid"
        );
    }

    #[test]
    fn test_multiple_despawns_and_recycles() {
        let mut world = World::new();

        let e1 = world.spawn_entity();
        let _e2 = world.spawn_entity();
        let e3 = world.spawn_entity();

        world.despawn_entity(&e1);
        world.despawn_entity(&e3);

        // Next spawn should take index 2 (last freed) or 0 depending on pop order
        // But either way, it must be a valid, non-stale handle
        let e4 = world.spawn_entity();
        assert!(world.is_valid(&e4));

        // Verify that the despawned handles are still invalid
        assert!(!world.is_valid(&e1));
        assert!(!world.is_valid(&e3));
    }

    #[test]
    fn test_attach_detach_on_valid_entities() {
        let mut world = World::new();
        let e = world.spawn_entity();

        // Test attach
        let res_attach = world.attach_component(&e, String::from("Data"));
        assert!(res_attach.is_ok());

        // Test detach
        let res_detach = world.detach_component::<String>(&e);
        assert!(res_detach.is_ok());
    }
}
