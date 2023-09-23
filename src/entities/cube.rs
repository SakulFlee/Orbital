use winit::event::VirtualKeyCode;

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    entities::{EmptyEntity, OneShotEntity},
};

#[derive(Default)]
pub struct Cube;

impl Cube {
    pub const TAG: &str = "Cube";
}

impl TEntity for Cube {
    fn get_entity_config(&self) -> EntityConfiguration {
        EntityConfiguration::new(Self::TAG, UpdateFrequency::Slow, false)
    }

    fn update(&mut self, delta_time: f64, input_handler: &InputHandler) -> EntityAction {
        log::debug!("Tick! d: {delta_time}ms");

        if input_handler.is_key_pressed(&VirtualKeyCode::Space) {
            // Note: [`UpdateFrequency::Slow`] means we have to hold down Space
            log::debug!("SPACE! We are going to SPACEEEEEEEE!");

            return EntityAction::Spawn(vec![
                Box::new(EmptyEntity::new("empty")),
                Box::new(OneShotEntity::new("one-shot")),
            ]);
        }

        EntityAction::Keep
    }
}
