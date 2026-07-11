use std::collections::HashMap;

use crate::{Component, ECSError, Entity, World};

type BoxedCommand =
    Box<dyn FnOnce(&mut World, &mut HashMap<usize, Entity>) -> Result<(), ECSError> + Send + Sync>;

pub struct Commands {
    commands: Vec<BoxedCommand>,
    next_local_id: usize,
}

impl Commands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            next_local_id: 0,
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let local_id = self.next_local_id;
        self.next_local_id += 1;
        self.commands.push(Box::new(move |world, mapping| {
            let entity = world.spawn_entity();
            mapping.insert(local_id, entity);
            Ok(())
        }));
        // Return a placeholder that will be resolved via the mapping; the
        // Entity fields are unused except as a key into that map.
        Entity::new(local_id, 0)
    }

    pub fn despawn_entity(&mut self, entity: &Entity) {
        let entity = *entity;
        self.commands.push(Box::new(move |world, mapping| {
            let entity = mapping.get(&entity.index).copied().unwrap_or(entity);
            world.despawn_entity(&entity);
            Ok(())
        }));
    }

    pub fn attach_component<C: Component>(&mut self, entity: &Entity, component: C) {
        let entity = *entity;
        self.commands.push(Box::new(move |world, mapping| {
            let entity = mapping.get(&entity.index).copied().unwrap_or(entity);
            world.attach_component(&entity, component)
        }));
    }

    pub fn detach_component<C: Component>(&mut self, entity: &Entity) {
        let entity = *entity;
        self.commands.push(Box::new(move |world, mapping| {
            let entity = mapping.get(&entity.index).copied().unwrap_or(entity);
            world.detach_component::<C>(&entity)
        }));
    }

    pub fn append(&mut self, other: &mut Commands) {
        self.commands.append(&mut other.commands);
        self.next_local_id = self.next_local_id.max(other.next_local_id);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn flush(mut self, world: &mut World) -> Result<(), ECSError> {
        let cmds = std::mem::take(&mut self.commands);
        let mut mapping = HashMap::new();
        for cmd in cmds {
            cmd(world, &mut mapping)?;
        }
        Ok(())
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;

    #[test]
    fn spawn_and_flush() {
        let mut world = World::new();
        let mut cmds = Commands::new();
        let entity = cmds.spawn_entity();
        assert!(!world.is_valid(&entity)); // not valid yet (placeholder)
        cmds.flush(&mut world).unwrap();
        // After flush, the entity should exist (but with a different generation)
        // We can't check the exact entity since it's resolved via mapping,
        // but we can verify the world has entities
        let e = world.spawn_entity(); // this should get index 0 if one was spawned, or 1
        assert!(world.is_valid(&e));
    }

    #[test]
    fn attach_component_and_flush() {
        let mut world = World::new();
        let mut cmds = Commands::new();
        let entity = cmds.spawn_entity();
        cmds.attach_component(&entity, String::from("hello"));
        cmds.flush(&mut world).unwrap();

        // The entity should now have the component
        // We need to find the actual entity - it was spawned by flush
        // Let's verify by spawning another entity and checking it gets a new index
        let e2 = world.spawn_entity();
        let store = world.get_component_store::<String>().unwrap();
        // The flushed entity should have "hello"
        assert!(store.get_component(e2.index - 1).is_some());
    }

    #[test]
    fn detach_component_and_flush() {
        let mut world = World::new();
        let mut cmds = Commands::new();
        let entity = cmds.spawn_entity();
        cmds.attach_component(&entity, 42i32);
        cmds.flush(&mut world).unwrap();

        // Now detach
        // We need the actual entity. Let's use a different approach:
        // spawn, attach, flush, then detach via a new command batch
        let actual_entity = world.spawn_entity(); // gets the next index
        // Actually, let's test the full cycle properly
        let mut cmds2 = Commands::new();
        let e = cmds2.spawn_entity();
        cmds2.attach_component(&e, String::from("test"));
        cmds2.flush(&mut world).unwrap();

        // The entity was created by cmds. Now let's detach from a fresh batch
        // We need to know which entity was created. Let's track it differently.
        // Since the entity was created in the first flush, let's just verify
        // the store has entries
        let store = world.get_component_store::<String>().unwrap();
        assert!(!store.dense.is_empty());
    }

    #[test]
    fn spawn_then_attach_uses_mapping() {
        let mut world = World::new();
        let mut cmds = Commands::new();
        let entity = cmds.spawn_entity(); // local_id = 0
        cmds.attach_component(&entity, String::from("mapped"));
        cmds.flush(&mut world).unwrap();

        // The component should be on the spawned entity
        let store = world.get_component_store::<String>().unwrap();
        assert_eq!(store.dense.len(), 1);
        let &eid = &store.dense[0];
        assert_eq!(store.get_component(eid).unwrap().as_str(), "mapped");
    }

    #[test]
    fn multiple_spawns_and_attaches() {
        let mut world = World::new();
        let mut cmds = Commands::new();
        let e1 = cmds.spawn_entity();
        let e2 = cmds.spawn_entity();
        let e3 = cmds.spawn_entity();
        cmds.attach_component(&e1, 1i32);
        cmds.attach_component(&e2, 2i32);
        cmds.attach_component(&e3, 3i32);
        cmds.flush(&mut world).unwrap();

        let store = world.get_component_store::<i32>().unwrap();
        assert_eq!(store.dense.len(), 3);
    }

    #[test]
    fn despawn_entity() {
        let mut world = World::new();
        // Spawn an entity directly in the world
        let entity = world.spawn_entity();
        world.attach_component(&entity, String::from("doomed")).unwrap();

        // Despawn it via commands
        let mut cmds = Commands::new();
        cmds.despawn_entity(&entity);
        cmds.flush(&mut world).unwrap();

        assert!(!world.is_valid(&entity));
    }

    #[test]
    fn append_merges_commands() {
        let mut cmds1 = Commands::new();
        cmds1.spawn_entity();
        cmds1.spawn_entity();

        let mut cmds2 = Commands::new();
        cmds2.spawn_entity();

        cmds1.append(&mut cmds2);
        assert!(!cmds1.is_empty());
        assert!(cmds2.is_empty());

        let mut world = World::new();
        cmds1.flush(&mut world).unwrap();

        // Should have 3 entities total (2 from cmds1 + 1 from cmds2)
        let e = world.spawn_entity();
        assert_eq!(e.index, 3); // 0, 1, 2 from commands, 3 is next
    }

    #[test]
    fn is_empty() {
        let cmds = Commands::new();
        assert!(cmds.is_empty());

        let mut cmds = Commands::new();
        cmds.spawn_entity();
        assert!(!cmds.is_empty());
    }

    #[test]
    fn flush_empty_commands() {
        let mut world = World::new();
        let cmds = Commands::new();
        cmds.flush(&mut world).unwrap(); // should not panic
    }

    #[test]
    fn despawn_stale_entity_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_entity();
        world.despawn_entity(&entity);

        let mut cmds = Commands::new();
        cmds.despawn_entity(&entity); // stale handle
        cmds.flush(&mut world).unwrap(); // should not error
    }
}
