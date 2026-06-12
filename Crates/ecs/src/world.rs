use crate::{ArchetypeManager, Component, ComponentStore, Entity, WorldComponentStorage};
use std::any::TypeId;
use std::collections::HashMap;

/// The world is the central hub for all entities and components.
#[derive(Debug)]
pub struct World {
    /// Manages archetypes and entity movement between them
    archetype_manager: ArchetypeManager,
    /// Stores components for each type
    component_stores: HashMap<TypeId, Box<dyn WorldComponentStorage>>,
    /// Counter for new entities
    next_entity_index: usize,
    /// Entity generation counter to prevent use-after-free
    generation: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetype_manager: ArchetypeManager::new(),
            component_stores: HashMap::new(),
            next_entity_index: 0,
            generation: 0,
        }
    }

    /// Spawns a new entity with no components.
    pub fn spawn_entity(&mut self) -> Entity {
        let index = self.next_entity_index;
        self.next_entity_index += 1;

        let entity = Entity::new(index, self.generation as usize);

        // Add to default archetype (empty)
        let archetype_idx = self.archetype_manager.get_or_create_archetype(&[]);
        self.archetype_manager.add_entity(entity, archetype_idx);

        entity
    }

    /// Removes an entity from the world.
    pub fn despawn_entity(&mut self, entity: &Entity) {
        if !self.is_valid(entity) {
            return;
        }

        // Remove entity from all component stores
        for _store in self.component_stores.values_mut() {
            _store.remove_entity(entity.index);
        }

        // Get the entity's location (archetype index and index within archetype)
        if let Some((archetype_idx, entity_idx_in_archetype)) =
            self.archetype_manager.get_entity_location(entity.index)
        {
            // Remove entity from archetype
            {
                let archetype = &mut self.archetype_manager.archetypes_mut()[archetype_idx];
                archetype.remove_entity(entity_idx_in_archetype);
                // If the archetype is now empty, remove it from the archetype map
                if archetype.is_empty() {
                    let mask = archetype.mask;
                    self.archetype_manager.archetype_map_mut().remove(&mask);
                }
            }
            // Remove the entity from the archetype manager's entity mapping
            self.archetype_manager.remove_entity(entity.index);
        }
    }

    /// Attaches a component to an entity.
    pub fn attach_component<T: Component>(
        &mut self,
        entity: &Entity,
        value: T,
    ) -> Result<(), &'static str> {
        if !self.is_valid(entity) {
            return Err("Invalid entity");
        }

        // Get the component type ID
        let type_id = TypeId::of::<T>();

        // Get current component types for the entity
        let current_types = self
            .archetype_manager
            .get_component_types_for_entity(entity.index)
            .cloned()
            .unwrap_or_else(|| Vec::new());

        // Check if the entity already has this component type
        if !current_types.contains(&type_id) {
            // Create a new set of component types including the new one
            let mut new_types = current_types.clone();
            new_types.push(type_id);
            // Move the entity to the new archetype
            self.archetype_manager.move_entity(*entity, &new_types);
        }

        // Get or create the component store for this type
        let store = self
            .component_stores
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStore::<T>::new()));

        // Downcast the store to ComponentStore<T> and attach the component
        if let Some(store) = store.as_any_mut().downcast_mut::<ComponentStore<T>>() {
            store.attach(entity.index, value);
        } else {
            // This should not happen because we just inserted a ComponentStore<T>
            return Err("Failed to downcast component store");
        }

        Ok(())
    }

    /// Detaches a component from an entity.
    pub fn detach_component<T: Component>(
        &mut self,
        entity: &Entity,
    ) -> Result<Option<T>, &'static str> {
        if !self.is_valid(entity) {
            return Err("Invalid entity");
        }

        // Check if entity has this component
        if !self.has_component::<T>(entity) {
            return Ok(None);
        }

        // Get the component type ID
        let type_id = TypeId::of::<T>();

        // Get current component types for the entity
        let current_types = self
            .archetype_manager
            .get_component_types_for_entity(entity.index)
            .cloned()
            .unwrap_or_else(|| Vec::new());

        // Create a new set of component types without the removed one
        let mut new_types = current_types.clone();
        new_types.retain(|&tid| tid != type_id);

        // Move the entity to the new archetype (which may be the empty archetype)
        self.archetype_manager.move_entity(*entity, &new_types);

        // Get the component store for this type and detach the component
        if let Some(store) = self.component_stores.get_mut(&type_id) {
            if let Some(store) = store.as_any_mut().downcast_mut::<ComponentStore<T>>() {
                let component = store.detach(entity.index);
                return Ok(component);
            }
        }

        // If we get here, the store doesn't exist or downcast failed
        Ok(None)
    }

    /// Gets a reference to a component on an entity.
    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        if !self.is_valid(entity) {
            return None;
        }

        // Get the component store for this type
        if let Some(store) = self.component_stores.get(&TypeId::of::<T>()) {
            // Downcast to ComponentStore<T> and get the component
            if let Some(store) = store.as_any().downcast_ref::<ComponentStore<T>>() {
                return store.get_component(entity.index);
            }
        }

        None
    }

    /// Checks if an entity has a specific component.
    pub fn has_component<T: Component>(&self, entity: &Entity) -> bool {
        if !self.is_valid(entity) {
            return false;
        }

        // Check if the entity is in the archetype manager
        if let Some(archetype_idx) = self.archetype_manager.get_archetype_index(entity.index) {
            let archetype = &self.archetype_manager.archetypes()[archetype_idx];
            archetype.contains_component_type(TypeId::of::<T>())
        } else {
            false
        }
    }

    /// Checks if an entity is valid (not despawned).
    pub fn is_valid(&self, entity: &Entity) -> bool {
        entity.generation == self.generation as usize
            && self
                .archetype_manager
                .get_archetype_index(entity.index)
                .is_some()
    }

    /// Gets the number of entities in the world.
    pub fn entity_count(&self) -> usize {
        self.archetype_manager
            .archetypes()
            .iter()
            .map(|a| a.len())
            .sum()
    }

    /// Clears all entities and components from the world.
    pub fn clear(&mut self) {
        // Remove all entities from archetypes (this will also update the entity mappings)
        for archetype in self.archetype_manager.archetypes_mut().iter_mut() {
            while !archetype.is_empty() {
                archetype.remove_entity(0);
            }
        }

        // Clear all component stores
        self.component_stores.clear();

        // Reset entity counter and generation
        self.next_entity_index = 0;
        self.generation += 1;
    }

    /// Gets the number of archetypes in the world.
    pub fn archetype_count(&self) -> usize {
        self.archetype_manager.archetypes().len()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation_and_despawn() {
        let mut world = World::new();

        // Create entities
        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();

        assert!(world.is_valid(&e1));
        assert!(world.is_valid(&e2));

        // Despawn entity
        world.despawn_entity(&e1);

        assert!(!world.is_valid(&e1));
        assert!(world.is_valid(&e2));
    }

    #[test]
    fn test_component_attachment() {
        let mut world = World::new();
        let e = world.spawn_entity();

        // Attach component (placeholder)
        let res = world.attach_component(&e, String::from("Data"));
        assert!(res.is_ok());

        // Get component back (placeholder - returns None for now)
        if let Some(comp) = world.get_component::<String>(&e) {
            assert_eq!(comp, &"Data");
        } else {
            panic!("Component not found!");
        }
    }

    #[test]
    fn test_component_detach() {
        let mut world = World::new();
        let e = world.spawn_entity();

        // Attach component (placeholder)
        world.attach_component(&e, String::from("Data")).unwrap();

        // Detach component (placeholder - returns None for now)
        if let Some(comp) = world.detach_component::<String>(&e).unwrap() {
            assert_eq!(comp, "Data");
        } else {
            panic!("Component not detached!");
        }

        // Verify component is gone
        assert!(!world.has_component::<String>(&e));
    }

    #[test]
    fn test_multiple_components() {
        let mut world = World::new();
        let e = world.spawn_entity();

        // Attach multiple components (placeholder)
        world.attach_component(&e, String::from("Name")).unwrap();
        world.attach_component(&e, i32::from(42)).unwrap();

        // Get both components (placeholder - returns None for now)
        if let Some(name) = world.get_component::<String>(&e) {
            assert_eq!(name, &"Name");
        } else {
            panic!("Name component not found!");
        }

        if let Some(value) = world.get_component::<i32>(&e) {
            assert_eq!(*value, 42);
        } else {
            panic!("Value component not found!");
        }
    }

    #[test]
    fn test_clear() {
        let mut world = World::new();
        let e1 = world.spawn_entity();
        let e2 = world.spawn_entity();
        world.attach_component(&e1, String::from("test")).unwrap();
        world.attach_component(&e2, 42).unwrap();

        assert!(world.is_valid(&e1));
        assert!(world.is_valid(&e2));
        assert!(world.has_component::<String>(&e1));
        assert!(world.has_component::<i32>(&e2));

        world.clear();

        assert!(!world.is_valid(&e1));
        assert!(!world.is_valid(&e2));
        assert!(!world.has_component::<String>(&e1));
        assert!(!world.has_component::<i32>(&e2));
        // After clearing, the world should be empty and we can spawn new entities
        let e3 = world.spawn_entity();
        assert!(world.is_valid(&e3));
    }
}
