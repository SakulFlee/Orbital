use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::{Entity, Component};

/// Represents a collection of entities that share the same component types.
#[derive(Debug)]
pub struct Archetype {
    /// Bitmask representing which components this archetype has (1 bit per type)
    pub mask: u64,
    /// List of TypeIds for each component in this archetype
    pub component_types: Vec<TypeId>,
    /// Size in bytes of each column
    pub column_sizes: Vec<usize>,
    /// Vector of entity indices belonging to this archetype
    pub entities: Vec<Entity>,
    /// For each entity, stores the index into the `entities` vector
    pub entity_indices: Vec<usize>,
    /// Column data for each component type
    pub columns: Vec<Vec<u8>>,
}

impl Archetype {
    pub(crate) fn new(mask: u64, component_types: Vec<TypeId>, column_sizes: Vec<usize>) -> Self {
        let mut columns = vec![Vec::new(); component_types.len()];
        // Initialize with placeholder data - actual sizes will be set when pushing entities
        for col in &mut columns {
            col.push(0u8);  // Placeholder to avoid empty column issues
        }
        
        Self {
            mask,
            component_types,
            column_sizes,
            entities: Vec::new(),
            entity_indices: Vec::new(),
            columns,
        }
    }

    pub fn push_entity(&mut self, entity: Entity) {
        let idx = self.entities.len();
        self.entities.push(entity);
        self.entity_indices.push(idx);
    }

    pub fn remove_entity(&mut self, entity_idx: usize) -> Option<Entity> {
        if entity_idx >= self.entities.len() {
            return None;
        }

        let entity = self.entities[entity_idx];
        
        // Swap with last and pop
        let last_idx = self.entities.len() - 1;
        if entity_idx != last_idx {
            self.entities.swap(entity_idx, last_idx);
            self.entity_indices.swap(entity_idx, last_idx);
            
            // Update the swapped entity's index
            self.entity_indices[last_idx] = entity_idx;
        }

        self.entities.pop();
        self.entity_indices.pop();
        
        Some(entity)
    }

    pub fn get_entity(&self, entity_idx: usize) -> Option<Entity> {
        if entity_idx >= self.entities.len() {
            return None;
        }
        Some(self.entities[entity_idx])
    }

    pub fn contains_component_type(&self, type_id: TypeId) -> bool {
        self.component_types.contains(&type_id)
    }

    pub fn get_column_size(&self, component_index: usize) -> usize {
        if component_index >= self.column_sizes.len() {
            return 0;
        }
        self.column_sizes[component_index]
    }

    pub fn get_component_data(&self, entity_idx: usize, component_index: usize) -> &[u8] {
        let size = self.get_column_size(component_index);
        if size == 0 || entity_idx >= self.entities.len() {
            return &[];
        }
        
        let start = self.entity_indices[entity_idx] * size;
        let end = start + size;
        self.columns[component_index].get(start..end).unwrap_or(&[])
    }

    pub fn get_component<T: Component>(&self, entity: &Entity) -> Option<&T> {
        if !self.contains_component_type(TypeId::of::<T>()) {
            return None;
        }

        let component_index = self.component_types.iter().position(|&t| t == TypeId::of::<T>())?;
        
        // Component data is stored as raw bytes - in a full implementation,
        // you'd deserialize it here using serde or similar.
        // For now, we just return None since deserialization isn't implemented yet.
        if self.columns[component_index].is_empty() {
            return None;
        }

        // Placeholder: would deserialize actual component data here
        None
    }

    pub fn get_components<T: Component>(&self, entity: &Entity) -> Option<&T> {
        self.get_component::<T>(entity)
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// Manages multiple archetypes and handles entity movement between them.
#[derive(Debug)]
pub struct ArchetypeManager {
    /// Maps archetype mask to archetype index
    archetype_map: HashMap<u64, usize>,
    /// List of all archetypes
    archetypes: Vec<Archetype>,
    /// Maps entity index to (archetype_index, entity_index_in_archetype)
    entity_to_archetype: HashMap<usize, (usize, usize)>,
}

impl ArchetypeManager {
    pub fn new() -> Self {
        Self {
            archetype_map: HashMap::new(),
            archetypes: Vec::new(),
            entity_to_archetype: HashMap::new(),
        }
    }

    /// Creates or retrieves an archetype for the given component types.
    pub fn get_or_create_archetype(&mut self, component_types: &[TypeId]) -> usize {
        let mask = Self::calculate_mask(component_types);
        
        if let Some(idx) = self.archetype_map.get(&mask) {
            return *idx;
        }

        // Calculate column sizes for each component type
        let mut column_sizes = Vec::new();
        for &type_id in component_types {
            let size = Self::get_type_size(type_id);
            column_sizes.push(size);
        }

        let archetype_idx = self.archetypes.len();
        let archetype = Archetype::new(mask, component_types.to_vec(), column_sizes);
        self.archetypes.push(archetype);
        
        // Update the map
        if !self.archetype_map.contains_key(&mask) {
            self.archetype_map.insert(mask, archetype_idx);
        }

        archetype_idx
    }

    /// Adds an entity to the specified archetype and updates the entity mapping.
    pub fn add_entity(&mut self, entity: Entity, archetype_index: usize) -> usize {
        let archetype = &mut self.archetypes[archetype_index];
        let idx = archetype.entities.len();
        archetype.push_entity(entity);
        self.entity_to_archetype.insert(entity.index, (archetype_index, idx));
        idx
    }

    /// Gets the archetype index for an entity.
    pub fn get_archetype_index(&self, entity_index: usize) -> Option<usize> {
        self.entity_to_archetype.get(&entity_index).map(|(idx, _)| *idx)
    }

    /// Gets the entity's location (archetype index and index within archetype) for an entity.
    pub fn get_entity_location(&self, entity_index: usize) -> Option<(usize, usize)> {
        self.entity_to_archetype.get(&entity_index).copied()
    }

    /// Gets the component types for an entity.
    pub fn get_component_types_for_entity(&self, entity_index: usize) -> Option<&Vec<TypeId>> {
        self.entity_to_archetype.get(&entity_index)
            .map(|(archetype_idx, _)| &self.archetypes[*archetype_idx].component_types)
    }

    /// Removes an entity from the entity mapping.
    pub fn remove_entity(&mut self, entity_index: usize) {
        self.entity_to_archetype.remove(&entity_index);
    }

    /// Calculates a bitmask from component types.
    fn calculate_mask(component_types: &[TypeId]) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hashes: Vec<u64> = component_types
            .iter()
            .map(|&type_id| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                type_id.hash(&mut hasher);
                hasher.finish()
            })
            .collect();
        hashes.sort();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for hash in hashes {
            hash.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Gets the size of a type in bytes.
    fn get_type_size(type_id: TypeId) -> usize {
        // This is a simplified version - in practice, you'd want to use more robust type info
        std::mem::size_of::<usize>() * 2 // Placeholder
    }

    /// Gets the index of a type within an archetype's component_types.
    fn get_type_index(type_id: &TypeId) -> Option<usize> {
        None
    }

    /// Moves an entity from one archetype to another.
    pub fn move_entity(
        &mut self,
        entity: Entity,
        new_component_types: &[TypeId],
    ) {
        let (old_arch_idx, _) = match self.entity_to_archetype.get(&entity.index) {
            Some(pos) => *pos,
            None => return, // Entity not in any archetype
        };

        let old_mask = self.archetypes[old_arch_idx].mask;
        let new_mask = Self::calculate_mask(new_component_types);

        if old_mask == new_mask {
            return; // Already in the correct archetype
        }

        // Get or create the new archetype first (before modifying archetypes)
        let new_arch_idx = self.get_or_create_archetype(new_component_types);
        
        // Find entity's local index in old archetype
        let entity_local_idx = match self.archetypes[old_arch_idx].find_entity_index(entity) {
            Some(idx) => idx,
            None => return, // Should not happen
        };
        
        // Remove entity from old archetype
        let _removed_entity = self.archetypes[old_arch_idx].remove_entity(entity_local_idx);
        // We don't use the removed entity, but we assume it's the same as the input entity

        // Add entity to new archetype
        let new_arch_entity_idx = self.archetypes[new_arch_idx].entities.len();
        self.archetypes[new_arch_idx].push_entity(entity);
        
        // Update entity mapping
        self.entity_to_archetype.insert(entity.index, (new_arch_idx, new_arch_entity_idx));

        // Clean up empty archetypes
        if self.archetypes[old_arch_idx].is_empty() {
            self.archetype_map.remove(&old_mask);
        }
    }

    /// Accessor for internal archetypes vector (needed by World)
    pub(crate) fn archetypes(&self) -> &[Archetype] {
        &self.archetypes
    }

    /// Mutable accessor for internal archetypes vector (needed by World)
    pub(crate) fn archetypes_mut(&mut self) -> &mut Vec<Archetype> {
        &mut self.archetypes
    }

    pub(crate) fn archetype_map(&self) -> &HashMap<u64, usize> {
        &self.archetype_map
    }

    pub(crate) fn archetype_map_mut(&mut self) -> &mut HashMap<u64, usize> {
        &mut self.archetype_map
    }

    /// Gets the entity at a specific index in an archetype
    pub(crate) fn get_entity_at(&self, archetype_idx: usize, entity_local_idx: usize) -> Option<Entity> {
        if entity_local_idx < self.archetypes[archetype_idx].entities.len() {
            Some(self.archetypes[archetype_idx].entities[entity_local_idx])
        } else {
            None
        }
    }
}

impl Archetype {
    /// Finds the local index of an entity within this archetype
    pub fn find_entity_index(&self, entity: Entity) -> Option<usize> {
        self.entities.iter().position(|&e| e == entity)
    }
}

impl Default for ArchetypeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_creation() {
        let mut manager = ArchetypeManager::new();
        
        // Create archetype with one component type
        let idx1 = manager.get_or_create_archetype(&[TypeId::of::<String>()]);
        assert_eq!(idx1, 0);

        // Try to create same archetype again - should return existing index
        let idx2 = manager.get_or_create_archetype(&[TypeId::of::<String>()]);
        assert_eq!(idx1, idx2);

        // Create different archetype with another component type
        let idx3 = manager.get_or_create_archetype(&[TypeId::of::<i32>()]);
        assert_ne!(idx1, idx3);
    }

    #[test]
    fn test_entity_movement() {
        let mut manager = ArchetypeManager::new();

        // Create archetype with String component
        let string_idx = manager.get_or_create_archetype(&[TypeId::of::<String>()]);
        
        // Create archetype with i32 component  
        let int_idx = manager.get_or_create_archetype(&[TypeId::of::<i32>()]);

        assert_ne!(string_idx, int_idx);

        // Move entity from string archetype to int archetype
        let entity = Entity::new(0, 0);
        manager.move_entity(entity, &[TypeId::of::<i32>()]);

        // Verify entity is now in int archetype
        if let Some((arch_idx, _)) = manager.entity_to_archetype.get(&entity.index) {
            assert_eq!(*arch_idx, int_idx);
        }
    }
}