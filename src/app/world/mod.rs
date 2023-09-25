mod entity;
pub use entity::*;

use wgpu::{Color, Device, Queue};

use crate::engine::TMesh;

use super::InputHandler;

pub struct World {
    clear_color: Color,
    entities: Vec<EntityContainer>,
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
        // TODO: Check for Tag being used multiple times
        let entity_container = EntityContainer::from_boxed_entity(entity);

        self.entities.push(entity_container);
    }

    pub fn remove_entity(&mut self, tag: &str) -> Option<BoxedEntity> {
        match self
            .entities
            .iter()
            .position(|container| container.is_tag(tag))
        {
            Some(index) => Some(self.entities.remove(index).get_and_move_entity()),
            None => None,
        }
    }

    pub fn has_entity(&self, tag: &str) -> bool {
        self.entities.iter().any(|container| container.is_tag(tag))
    }

    pub fn get_entity(&self, tag: &str) -> Option<&BoxedEntity> {
        self.entities
            .iter()
            .find(|container| container.is_tag(tag))
            .map(|container| container.get_entity())
    }

    pub fn get_entity_mut(&mut self, tag: &str) -> Option<&mut BoxedEntity> {
        self.entities
            .iter_mut()
            .find(|container| container.is_tag(tag))
            .map(|container| container.get_entity_mut())
    }

    pub fn get_updateable(&self, frequency: UpdateFrequency) -> Vec<&BoxedEntity> {
        if frequency == UpdateFrequency::None {
            return vec![];
        }

        self.entities
            .iter()
            .filter(|container| {
                *container.get_entity_configuration().get_update_frequency() == frequency
            })
            .map(|container| container.get_entity())
            .collect()
    }

    pub fn get_updateable_mut(&mut self, frequency: UpdateFrequency) -> Vec<&mut BoxedEntity> {
        if frequency == UpdateFrequency::None {
            return vec![];
        }

        self.entities
            .iter_mut()
            .filter(|container| {
                *container.get_entity_configuration().get_update_frequency() == frequency
            })
            .map(|container| container.get_entity_mut())
            .collect()
    }

    pub fn get_prepared_renderable(&self) -> Vec<&BoxedEntity> {
        self.entities
            .iter()
            .filter(|container| {
                container.is_prepared() && container.get_entity_configuration().get_do_render()
            })
            .map(|container| container.get_entity())
            .collect()
    }

    pub fn get_unprepared_renderable(&mut self) -> Vec<&mut EntityContainer> {
        self.entities
            .iter_mut()
            .filter(|container| {
                !container.is_prepared() && container.get_entity_configuration().get_do_render()
            })
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

    pub fn prepare_render_and_collect_meshes(
        &mut self,
        device: &Device,
        queue: &Queue,
    ) -> Vec<&dyn TMesh> {
        // Prepare rendere where needed
        self.get_unprepared_renderable()
            .iter_mut()
            .for_each(|x| x.prepare_entity(device, queue));

        // Retrieve meshes
        self.get_prepared_renderable()
            .iter()
            .flat_map(|x| x.get_meshes())
            .collect::<Vec<_>>()
    }
}
