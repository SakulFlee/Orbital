mod entity;
pub use entity::*;

pub struct World {
    entities: Vec<(EntityConfiguration, BoxedEntity)>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: BoxedEntity) {
        self.entities.push((entity.get_entity_config(), entity));
    }

    pub fn remove_entity(&mut self, tag: &str) -> Option<BoxedEntity> {
        match self
            .entities
            .iter()
            .position(|(config, entity)| config.get_tag() == tag)
        {
            Some(index) => Some(self.entities.remove(index).1),
            None => None,
        }
    }

    pub fn has_entity(&self, tag: &str) -> bool {
        self.entities
            .iter()
            .any(|(config, entity)| config.get_tag() == tag)
    }

    pub fn get_entity(&self, tag: &str) -> Option<&BoxedEntity> {
        self.entities
            .iter()
            .find(|(config, entry)| config.get_tag() == tag)
            .map(|(_, entity)| entity)
    }

    pub fn get_entity_mut(&mut self, tag: &str) -> Option<&mut BoxedEntity> {
        self.entities
            .iter_mut()
            .find(|(config, entry)| config.get_tag() == tag)
            .map(|(_, entity)| entity)
    }

    pub fn get_updateable(&self, frequency: UpdateFrequency) -> Vec<&BoxedEntity> {
        if frequency == UpdateFrequency::None {
            return vec![];
        }

        self.entities
            .iter()
            .filter(|(config, _)| *config.get_update_frequency() == frequency)
            .map(|(_, entity)| entity)
            .collect()
    }

    pub fn get_updateable_mut(&mut self, frequency: UpdateFrequency) -> Vec<&mut BoxedEntity> {
        if frequency == UpdateFrequency::None {
            return vec![];
        }

        self.entities
            .iter_mut()
            .filter(|(config, _)| *config.get_update_frequency() == frequency)
            .map(|(_, entity)| entity)
            .collect()
    }

    pub fn get_renderable(&self) -> Vec<&BoxedEntity> {
        self.entities
            .iter()
            .filter(|(config, _)| config.get_render())
            .map(|(_, entity)| entity)
            .collect()
    }
}
