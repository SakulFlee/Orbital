use wgpu::{Device, Queue};

use super::{BoxedEntity, EntityConfiguration};

pub struct EntityContainer {
    entity_configuration: EntityConfiguration,
    entity: BoxedEntity,
    is_prepared: bool,
}

impl EntityContainer {
    pub fn from_boxed_entity(entity: BoxedEntity) -> Self {
        let entity_configuration = entity.get_entity_configuration();
        Self {
            entity_configuration,
            entity,
            is_prepared: false,
        }
    }

    pub fn prepare_entity(&mut self, device: &Device, queue: &Queue) {
        self.entity.prepare_render(device, queue);
        self.is_prepared = true;
    }

    pub fn get_entity_configuration(&self) -> &EntityConfiguration {
        &self.entity_configuration
    }

    pub fn get_entity(&self) -> &BoxedEntity {
        &self.entity
    }

    pub fn get_entity_mut(&mut self) -> &mut BoxedEntity {
        &mut self.entity
    }

    pub fn get_and_move_entity(self) -> BoxedEntity {
        self.entity
    }

    pub fn is_prepared(&self) -> bool {
        self.is_prepared
    }

    pub fn is_tag(&self, tag: &str) -> bool {
        self.entity_configuration.get_tag() == tag
    }
}
