use crate::{Component, Entity, ECSError, World};

type BoxedCommand = Box<dyn FnOnce(&mut World) -> Result<(), ECSError> + Send>;

pub struct Commands {
    commands: Vec<BoxedCommand>,
}

impl Commands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let entity = Entity::new(0, 0);
        self.commands.push(Box::new(move |world| {
            world.spawn_entity();
            Ok(())
        }));
        entity
    }

    pub fn despawn_entity(&mut self, entity: &Entity) {
        let entity = *entity;
        self.commands.push(Box::new(move |world| {
            world.despawn_entity(&entity);
            Ok(())
        }));
    }

    pub fn attach_component<C: Component>(&mut self, entity: &Entity, component: C) {
        let entity = *entity;
        self.commands.push(Box::new(move |world| {
            world.attach_component(&entity, component)
        }));
    }

    pub fn detach_component<C: Component>(&mut self, entity: &Entity) {
        let entity = *entity;
        self.commands.push(Box::new(move |world| {
            world.detach_component::<C>(&entity)
        }));
    }

    pub fn append(&mut self, other: &mut Commands) {
        self.commands.append(&mut other.commands);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn flush(&mut self, world: &mut World) -> Result<(), ECSError> {
        let cmds = std::mem::take(&mut self.commands);
        for cmd in cmds {
            cmd(world)?;
        }
        Ok(())
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self::new()
    }
}
