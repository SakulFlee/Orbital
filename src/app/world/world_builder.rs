use wgpu::{Color, Device, Queue};

use crate::engine::StandardAmbientLight;

use super::{BoxedEntity, EntityTagDuplicationBehaviour, World};

pub struct WorldBuilder {
    clear_color: Option<Color>,
    entity_tag_duplication_behaviour: Option<EntityTagDuplicationBehaviour>,
    entities: Vec<BoxedEntity>,
    ambient_light: Option<((f32, f32, f32), f32)>,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            clear_color: None,
            entity_tag_duplication_behaviour: None,
            entities: vec![],
            ambient_light: None,
        }
    }

    pub fn build(self, device: &Device, queue: &Queue) -> World {
        let ambient_light_raw = self.ambient_light.unwrap_or(((1.0, 1.0, 1.0), 0.1));
        let ambient_light = StandardAmbientLight::new(
            device,
            queue,
            ambient_light_raw.0.into(),
            ambient_light_raw.1,
        );

        let mut world = World {
            clear_color: self.clear_color.unwrap_or(Color::BLACK),
            entity_tag_duplication_behaviour: self
                .entity_tag_duplication_behaviour
                .unwrap_or(EntityTagDuplicationBehaviour::WarnOnDuplication),
            entities: vec![],
            ambient_light: ambient_light,
        };

        for entity in self.entities {
            world.add_entity(entity);
        }

        world
    }

    pub fn with_clear_color(mut self, color: Color) -> Self {
        self.clear_color = Some(color);
        self
    }

    pub fn with_entity_tag_duplication_behaviour(
        mut self,
        entity_tag_duplication_behaviour: EntityTagDuplicationBehaviour,
    ) -> Self {
        self.entity_tag_duplication_behaviour = Some(entity_tag_duplication_behaviour);
        self
    }

    pub fn with_entities(mut self, entities: Vec<BoxedEntity>) -> Self {
        self.entities.extend(entities);
        self
    }

    pub fn with_ambient_light(mut self, color: (f32, f32, f32), strength: f32) -> Self {
        self.ambient_light = Some((color, strength));
        self
    }
}
