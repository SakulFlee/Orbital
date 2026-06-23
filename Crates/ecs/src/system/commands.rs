use std::collections::HashMap;

use crate::{Component, Entity, ECSError, World};

type BoxedCommand = Box<dyn FnOnce(&mut World, &mut HashMap<usize, Entity>) -> Result<(), ECSError> + Send>;

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

    pub fn flush(&mut self, world: &mut World) -> Result<(), ECSError> {
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
