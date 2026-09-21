use std::collections::VecDeque;

use crate::{Entity, World};

/// Marker component indicating this entity has a parent.
/// The parent must be a valid, alive entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parent(pub Entity);

/// Component holding the children of this entity.
/// Children are stored in insertion order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Children(pub Vec<Entity>);

impl Children {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_child(mut self, child: Entity) -> Self {
        self.0.push(child);
        self
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}

/// Adds a child entity to a parent entity.
/// This sets the child's `Parent` component and adds the child to the parent's `Children`.
pub fn add_child(
    world: &mut World,
    parent: &Entity,
    child: &Entity,
) -> Result<(), crate::ECSError> {
    if !world.is_valid(parent) {
        return Err(crate::ECSError::InvalidEntity(*parent));
    }
    if !world.is_valid(child) {
        return Err(crate::ECSError::InvalidEntity(*child));
    }

    // Set child's parent
    world.attach_component(child, Parent(*parent))?;

    // Check if parent already has Children component
    let parent_has_children = world
        .get_component_store::<Children>()
        .map(|s| s.get_component(parent.index).is_some())
        .unwrap_or(false);

    if parent_has_children {
        // Add child to existing children list
        if let Some(store) = world.get_component_store_mut::<Children>() {
            let store_mut = store.get_mut_store();
            if let Some(idx) = store_mut.sparse.get(parent.index).and_then(|x| *x) {
                store_mut.components[idx].0.push(*child);
            }
        }
    } else {
        // Create new Children component with this child
        world.attach_component(parent, Children::new().with_child(*child))?;
    }

    Ok(())
}

/// Removes a child entity from its parent.
/// This removes the child's `Parent` component and removes the child from the parent's `Children`.
pub fn remove_child(
    world: &mut World,
    parent: &Entity,
    child: &Entity,
) -> Result<(), crate::ECSError> {
    if !world.is_valid(parent) {
        return Err(crate::ECSError::InvalidEntity(*parent));
    }

    // Remove child from parent's children list
    if let Some(store) = world.get_component_store_mut::<Children>() {
        let store_mut = store.get_mut_store();
        if let Some(idx) = store_mut.sparse.get(parent.index).and_then(|x| *x) {
            store_mut.components[idx].0.retain(|e| e != child);
        }
    }

    // Remove child's parent component
    let _ = world.detach_component::<Parent>(child);

    Ok(())
}

/// Returns the root entity (the entity with no parent) for a given entity.
/// If the entity has no parent, returns itself.
pub fn find_root(world: &World, entity: &Entity) -> Entity {
    let mut current = *entity;
    while let Some(store) = world.get_component_store::<Parent>() {
        if let Some(parent) = store.get_component(current.index) {
            if world.is_valid(&parent.0) {
                current = parent.0;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    current
}

/// Returns all ancestors of an entity, from parent to root.
/// Does not include the entity itself.
pub fn ancestors(world: &World, entity: &Entity) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut current = *entity;

    while let Some(store) = world.get_component_store::<Parent>() {
        if let Some(parent) = store.get_component(current.index) {
            if world.is_valid(&parent.0) {
                result.push(parent.0);
                current = parent.0;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}

/// Returns all descendants of an entity (children, grandchildren, etc.).
/// Uses BFS traversal.
pub fn descendants(world: &World, entity: &Entity) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut queue = VecDeque::new();

    // Start with direct children
    if let Some(store) = world.get_component_store::<Children>()
        && let Some(children) = store.get_component(entity.index)
    {
        for &child in &children.0 {
            if world.is_valid(&child) {
                queue.push_back(child);
                result.push(child);
            }
        }
    }

    // BFS through all descendants
    while let Some(current) = queue.pop_front() {
        if let Some(store) = world.get_component_store::<Children>()
            && let Some(children) = store.get_component(current.index)
        {
            for &child in &children.0 {
                if world.is_valid(&child) {
                    queue.push_back(child);
                    result.push(child);
                }
            }
        }
    }

    result
}

/// Cleans up stale parent/child references.
/// Removes entities from `Children` lists if the child is no longer valid,
/// and removes `Parent` components if the parent is no longer valid.
pub fn clean_hierarchy(world: &mut World) {
    // First, clean up parent references
    let stale_parents: Vec<Entity> = {
        let mut stale = Vec::new();
        if let Some(store) = world.get_component_store::<Parent>() {
            for &entity_idx in &store.dense {
                if let Some(parent) = store.get_component(entity_idx)
                    && !world.is_valid(&parent.0)
                {
                    let generation = world.generation(entity_idx);
                    stale.push(Entity::new(entity_idx, generation));
                }
            }
        }
        stale
    };

    for entity in stale_parents {
        let _ = world.detach_component::<Parent>(&entity);
    }

    // Then, clean up children lists
    if let Some(store) = world.get_component_store_mut::<Children>() {
        let store_mut = store.get_mut_store();
        let entities_to_check: Vec<usize> = store_mut.dense.clone();
        for entity_idx in entities_to_check {
            if let Some(idx) = store_mut.sparse.get(entity_idx).and_then(|x| *x) {
                store_mut.components[idx]
                    .0
                    .retain(|child| world.is_valid(child));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_children_components() {
        let mut world = World::new();
        let parent = world.spawn_entity();
        let child = world.spawn_entity();

        add_child(&mut world, &parent, &child).unwrap();

        // Check child has Parent component
        let store = world.get_component_store::<Parent>().unwrap();
        let parent_comp = store.get_component(child.index).unwrap();
        assert_eq!(parent_comp.0, parent);

        // Check parent has Children component
        let store = world.get_component_store::<Children>().unwrap();
        let children = store.get_component(parent.index).unwrap();
        assert_eq!(children.0.len(), 1);
        assert_eq!(children.0[0], child);
    }

    #[test]
    fn test_add_multiple_children() {
        let mut world = World::new();
        let parent = world.spawn_entity();
        let child1 = world.spawn_entity();
        let child2 = world.spawn_entity();
        let child3 = world.spawn_entity();

        add_child(&mut world, &parent, &child1).unwrap();
        add_child(&mut world, &parent, &child2).unwrap();
        add_child(&mut world, &parent, &child3).unwrap();

        let store = world.get_component_store::<Children>().unwrap();
        let children = store.get_component(parent.index).unwrap();
        assert_eq!(children.0.len(), 3);
        assert_eq!(children.0, vec![child1, child2, child3]);
    }

    #[test]
    fn test_remove_child() {
        let mut world = World::new();
        let parent = world.spawn_entity();
        let child = world.spawn_entity();

        add_child(&mut world, &parent, &child).unwrap();
        remove_child(&mut world, &parent, &child).unwrap();

        // Child should no longer have Parent component
        let store = world.get_component_store::<Parent>().unwrap();
        assert!(store.get_component(child.index).is_none());

        // Parent should have empty children list
        let store = world.get_component_store::<Children>().unwrap();
        let children = store.get_component(parent.index).unwrap();
        assert!(children.is_empty());
    }

    #[test]
    fn test_find_root() {
        let mut world = World::new();
        let root = world.spawn_entity();
        let child = world.spawn_entity();
        let grandchild = world.spawn_entity();

        add_child(&mut world, &root, &child).unwrap();
        add_child(&mut world, &child, &grandchild).unwrap();

        assert_eq!(find_root(&world, &grandchild), root);
        assert_eq!(find_root(&world, &child), root);
        assert_eq!(find_root(&world, &root), root);
    }

    #[test]
    fn test_ancestors() {
        let mut world = World::new();
        let root = world.spawn_entity();
        let child = world.spawn_entity();
        let grandchild = world.spawn_entity();

        add_child(&mut world, &root, &child).unwrap();
        add_child(&mut world, &child, &grandchild).unwrap();

        let anc = ancestors(&world, &grandchild);
        assert_eq!(anc, vec![child, root]);
    }

    #[test]
    fn test_descendants() {
        let mut world = World::new();
        let root = world.spawn_entity();
        let child1 = world.spawn_entity();
        let child2 = world.spawn_entity();
        let grandchild = world.spawn_entity();

        add_child(&mut world, &root, &child1).unwrap();
        add_child(&mut world, &root, &child2).unwrap();
        add_child(&mut world, &child1, &grandchild).unwrap();

        let desc = descendants(&world, &root);
        assert!(desc.contains(&child1));
        assert!(desc.contains(&child2));
        assert!(desc.contains(&grandchild));
        assert_eq!(desc.len(), 3);
    }

    #[test]
    fn test_clean_hierarchy_removes_stale_parent() {
        let mut world = World::new();
        let parent = world.spawn_entity();
        let child = world.spawn_entity();

        add_child(&mut world, &parent, &child).unwrap();

        // Despawn the parent
        world.despawn_entity(&parent);

        // Clean hierarchy should remove the stale Parent component
        clean_hierarchy(&mut world);

        let store = world.get_component_store::<Parent>().unwrap();
        assert!(store.get_component(child.index).is_none());
    }

    #[test]
    fn test_clean_hierarchy_removes_stale_children() {
        let mut world = World::new();
        let parent = world.spawn_entity();
        let child1 = world.spawn_entity();
        let child2 = world.spawn_entity();

        add_child(&mut world, &parent, &child1).unwrap();
        add_child(&mut world, &parent, &child2).unwrap();

        // Despawn one child
        world.despawn_entity(&child1);

        // Clean hierarchy should remove the stale child from parent's Children
        clean_hierarchy(&mut world);

        let store = world.get_component_store::<Children>().unwrap();
        let children = store.get_component(parent.index).unwrap();
        assert_eq!(children.0.len(), 1);
        assert_eq!(children.0[0], child2);
    }

    #[test]
    fn test_invalid_parent_entity() {
        let mut world = World::new();
        let child = world.spawn_entity();
        let fake_parent = Entity::new(999, 0);

        let result = add_child(&mut world, &fake_parent, &child);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_child_entity() {
        let mut world = World::new();
        let parent = world.spawn_entity();
        let fake_child = Entity::new(999, 0);

        let result = add_child(&mut world, &parent, &fake_child);
        assert!(result.is_err());
    }
}
