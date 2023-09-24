mod entity;

pub use entity::*;
use wgpu::Color;

use super::InputHandler;

pub struct World {
    clear_color: Color,
    entities: Vec<(EntityConfiguration, BoxedEntity)>,
}

impl World {
    pub const SKY_BLUE_ISH_COLOR: Color = Color {
        r: 0.0,
        g: 0.61176,
        b: 0.77647,
        a: 1.0,
    };

    pub fn new() -> Self {
        Self {
            clear_color: Color::BLACK,
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
            .position(|(config, _)| config.get_tag() == tag)
        {
            Some(index) => Some(self.entities.remove(index).1),
            None => None,
        }
    }

    pub fn has_entity(&self, tag: &str) -> bool {
        self.entities
            .iter()
            .any(|(config, _)| config.get_tag() == tag)
    }

    pub fn get_entity(&self, tag: &str) -> Option<&BoxedEntity> {
        self.entities
            .iter()
            .find(|(config, _)| config.get_tag() == tag)
            .map(|(_, entity)| entity)
    }

    pub fn get_entity_mut(&mut self, tag: &str) -> Option<&mut BoxedEntity> {
        self.entities
            .iter_mut()
            .find(|(config, _)| config.get_tag() == tag)
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

    pub fn get_clear_color(&self) -> Color {
        self.clear_color
    }

    pub fn call_updateable(
        &mut self,
        frequency: UpdateFrequency,
        delta_time: f64,
        input_handler: &InputHandler,
    ) {
        let entity_actions = self
            .get_updateable_mut(frequency)
            .iter_mut()
            .map(|x| x.update(delta_time, input_handler))
            .filter(|x| *x != EntityAction::Keep)
            .collect::<Vec<_>>();

        for entity_action in entity_actions {
            match entity_action {
                EntityAction::ClearColorAdjustment(color) => {
                    self.clear_color = color;
                }
                EntityAction::Spawn(entities) => {
                    for entity in entities {
                        self.add_entity(entity);
                    }
                }
                EntityAction::Remove(tags) => {
                    for tag in tags {
                        self.remove_entity(&tag);
                    }
                }
                EntityAction::Keep => (),
            }
        }
    }

    pub fn call_renderables(&self) {
        self.get_renderable().iter().for_each(|x| x.render());
    }
}
