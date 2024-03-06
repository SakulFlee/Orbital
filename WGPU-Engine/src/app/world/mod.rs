use wgpu::Color;

use crate::engine::{Camera, LogicalDevice, StandardAmbientLight, StandardPointLight, TMesh};

use super::InputHandler;

mod world_builder;
pub use world_builder::*;

mod entity;
pub use entity::*;

mod entity_tag_duplication_behaviour;
pub use entity_tag_duplication_behaviour::*;

pub struct World {
    clear_color: Color,
    entity_tag_duplication_behaviour: EntityTagDuplicationBehaviour,
    entities: Vec<EntityContainer>,
    ambient_light: StandardAmbientLight,
    point_lights: [StandardPointLight; 4],
}

impl World {
    pub const SKY_BLUE_ISH_COLOR: Color = Color {
        r: 0.0,
        g: 0.61176,
        b: 0.77647,
        a: 1.0,
    };

    pub fn from_builder(builder: WorldBuilder, logical_device: &LogicalDevice) -> Self {
        builder.build(logical_device)
    }

    pub fn add_entity(&mut self, entity: BoxedEntity) {
        let entity_container = EntityContainer::from_boxed_entity(entity);

        match self.entity_tag_duplication_behaviour {
            EntityTagDuplicationBehaviour::AllowDuplication => {
                // No special behaviour, just add
                self.entities.push(entity_container);
            }
            EntityTagDuplicationBehaviour::WarnOnDuplication => {
                // Warn if the tag exists, spawn otherwise
                if self.has_entity(entity_container.entity_configuration().tag()) {
                    log::warn!(
                        "Entity with a duplicated tag '{}' added!",
                        entity_container.entity_configuration().tag()
                    );
                }

                self.entities.push(entity_container);
            }
            EntityTagDuplicationBehaviour::PanicOnDuplication => {
                // Panic if the tag exists, spawn otherwise
                if self.has_entity(entity_container.entity_configuration().tag()) {
                    panic!(
                        "Entity with a duplicated tag '{}' added!",
                        entity_container.entity_configuration().tag()
                    );
                }

                self.entities.push(entity_container);
            }
            EntityTagDuplicationBehaviour::IgnoreEntityOnDuplication => {
                // Only spawn the entity if the tag isn't used yet
                if !self.has_entity(entity_container.entity_configuration().tag()) {
                    self.entities.push(entity_container);
                }
            }
            EntityTagDuplicationBehaviour::OverwriteEntityOnDuplication => {
                // If the entity tag already exists remove it, then spawn the new entity, otherwise just spawn the entity
                if self.has_entity(entity_container.entity_configuration().tag()) {
                    self.remove_entity(entity_container.entity_configuration().tag());
                }

                self.entities.push(entity_container);
            }
        }
    }

    pub fn remove_entity(&mut self, tag: &str) -> Option<BoxedEntity> {
        match self
            .entities
            .iter()
            .position(|container| container.is_tag(tag))
        {
            Some(index) => Some(self.entities.remove(index).and_move_entity()),
            None => None,
        }
    }

    pub fn has_entity(&self, tag: &str) -> bool {
        self.entities.iter().any(|container| container.is_tag(tag))
    }

    pub fn entity(&self, tag: &str) -> Option<&BoxedEntity> {
        self.entities
            .iter()
            .find(|container| container.is_tag(tag))
            .map(|container| container.entity())
    }

    pub fn entity_mut(&mut self, tag: &str) -> Option<&mut BoxedEntity> {
        self.entities
            .iter_mut()
            .find(|container| container.is_tag(tag))
            .map(|container| container.entity_mut())
    }

    pub fn updateable(&self, frequency: UpdateFrequency) -> Vec<&BoxedEntity> {
        if frequency == UpdateFrequency::None {
            return vec![];
        }

        self.entities
            .iter()
            .filter(|container| *container.entity_configuration().update_frequency() == frequency)
            .map(|container| container.entity())
            .collect()
    }

    pub fn updateable_mut(&mut self, frequency: UpdateFrequency) -> Vec<&mut BoxedEntity> {
        if frequency == UpdateFrequency::None {
            return vec![];
        }

        self.entities
            .iter_mut()
            .filter(|container| *container.entity_configuration().update_frequency() == frequency)
            .map(|container| container.entity_mut())
            .collect()
    }

    pub fn prepared_renderable(&self) -> Vec<&BoxedEntity> {
        self.entities
            .iter()
            .filter(|container| {
                container.is_prepared() && container.entity_configuration().do_render()
            })
            .map(|container| container.entity())
            .collect()
    }

    pub fn unprepared_renderable(&mut self) -> Vec<&mut EntityContainer> {
        self.entities
            .iter_mut()
            .filter(|container| {
                !container.is_prepared() && container.entity_configuration().do_render()
            })
            .collect()
    }

    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    pub fn call_updateable(
        &mut self,
        frequency: UpdateFrequency,
        delta_time: f64,
        input_handler: &InputHandler,
        camera: &mut Camera,
        logical_device: &LogicalDevice,
    ) {
        let entity_actions = self
            .updateable_mut(frequency)
            .iter_mut()
            .flat_map(|x| x.update(delta_time, input_handler))
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
                EntityAction::CameraChange(camera_change) => {
                    println!("{delta_time}");
                    camera.apply_camera_change(delta_time, logical_device, camera_change);
                }
                EntityAction::Keep => (),
            }
        }
    }

    pub fn prepare_render_and_collect_data(
        &mut self,
        logical_device: &LogicalDevice,
    ) -> (
        Vec<&dyn TMesh>,
        &StandardAmbientLight,
        &[StandardPointLight; 4],
    ) {
        // Prepare rendere where needed
        self.unprepared_renderable()
            .iter_mut()
            .for_each(|x| x.prepare_entity(logical_device));

        // Retrieve meshes
        (
            self.prepared_renderable()
                .iter()
                .flat_map(|x| x.meshes())
                .collect::<Vec<_>>(),
            &self.ambient_light,
            &self.point_lights,
        )
    }
}
