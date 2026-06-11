use std::any::TypeId;
use crate::{ArchetypeManager, Component, Entity};

/// The world is the central hub for all entities and components.
#[derive(Debug)]
pub struct World {
    /// Manages archetypes and entity movement between them
    archetype_manager: ArchetypeManager,
    /// Tracks which archetype each entity belongs to (entity_index -> archetype_index)
    entity_to_archetype: std::collections::HashMap<usize, usize>,
    /// Counter for new entities
    next_entity_index: usize,
    /// Entity generation counter to prevent use-after-free
    generation: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetype_manager: ArchetypeManager::new(),
            entity_to_archetype: std::collections::HashMap::new(),
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
        {
            let archetype = &mut self.archetype_manager.archetypes_mut()[archetype_idx];
            archetype.push_entity(entity);
        }

        // Update mapping
        self.entity_to_archetype.insert(index, archetype_idx);

        entity
    }

    /// Removes an entity from the world.
    pub fn despawn_entity(&mut self, entity: &Entity) {
        if !self.is_valid(entity) {
            return;
        }

        let archetype_idx = *self.entity_to_archetype.get(&entity.index).unwrap();
        
        // Get mutable access to the specific archetype
        let archetype = &mut self.archetype_manager.archetypes_mut()[*self.entity_to_archetype.get(&entity.index).unwrap()];
        
        // Get the mask before removing entity
        let archetype_mask = archetype.mask;
        
        // Remove entity from archetype
        archetype.remove_entity(entity.index);
        
        // Check if empty and remove from map (separate borrow)
        if archetype.is_empty() {
            self.archetype_manager.archetype_map_mut().remove(&archetype_mask);
        }

        // Remove from mapping
        self.entity_to_archetype.remove(&entity.index);
    }

    /// Attaches a component to an entity.
    pub fn attach_component<T: Component>(&mut self, _entity: &Entity, _value: T) -> Result<(), &'static str> {
        if !self.is_valid(_entity) {
            return Err("Invalid entity");
        }

        // TODO: Implement actual component attachment with archetype migration
        // This would involve:
        // 1. Getting current archetype for entity
        // 2. Creating new archetype with additional component type
        // 3. Moving entity to new archetype
        // 4. Copying existing component data
        // 5. Adding new component data
        
        Ok(())
    }

    /// Detaches a component from an entity.
    pub fn detach_component<T: Component>(&mut self, _entity: &Entity) -> Result<Option<T>, &'static str> {
        if !self.is_valid(_entity) {
            return Err("Invalid entity");
        }

        // Check if entity has this component
        if !self.has_component::<T>(_entity) {
            return Ok(None);
        }

        // TODO: Implement actual component detachment with archetype migration
        // This would involve:
        // 1. Getting current archetype for entity
        // 2. Creating new archetype without the removed component type
        // 3. Moving entity to new archetype
        // 4. Copying existing component data (excluding removed type)
        
        Ok(None)
    }

    /// Gets a reference to a component on an entity.
    pub fn get_component<T: Component>(&self, _entity: &Entity) -> Option<&T> {
        if !self.is_valid(_entity) {
            return None;
        }

        // TODO: Implement actual component retrieval from archetype columns
        // This would involve:
        // 1. Getting current archetype for entity
        // 2. Finding the column index for this component type
        // 3. Deserializing component data from bytes
        
        None
    }

    /// Checks if an entity has a specific component.
    pub fn has_component<T: Component>(&self, entity: &Entity) -> bool {
        if !self.is_valid(entity) {
            return false;
        }

        let archetype_idx = *self.entity_to_archetype.get(&entity.index).unwrap();
        let archetype = &self.archetype_manager.archetypes()[archetype_idx];

        archetype.contains_component_type(TypeId::of::<T>())
    }

    /// Checks if an entity is valid (not despawned).
    pub fn is_valid(&self, entity: &Entity) -> bool {
        entity.generation == self.generation as usize &&
        self.entity_to_archetype.contains_key(&entity.index)
    }

    /// Gets the number of entities in the world.
    pub fn entity_count(&self) -> usize {
        self.archetype_manager.archetypes().iter().map(|a| a.len()).sum()
    }

    /// Clears all entities and components from the world.
    pub fn clear(&mut self) {
        // Remove all entities from archetypes
        for archetype in self.archetype_manager.archetypes_mut().iter_mut() {
            while !archetype.is_empty() {
                archetype.remove_entity(0);
            }
        }

        // Clear mappings and counters
        self.entity_to_archetype.clear();
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
}
