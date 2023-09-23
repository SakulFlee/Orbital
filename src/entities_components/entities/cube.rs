use winit::event::VirtualKeyCode;

use crate::app::{EntityConfiguration, InputHandler, TEntity, UpdateFrequency};

#[derive(Default)]
pub struct Cube;

impl TEntity for Cube {
    fn get_entity_config(&self) -> EntityConfiguration {
        EntityConfiguration::new("Cube", UpdateFrequency::Slow, false)
    }

    fn update(&mut self, delta_time: f64, input_handler: &InputHandler) {
        log::debug!("Tick! d: {delta_time}ms");

        if input_handler.is_key_pressed(&VirtualKeyCode::Space) {
            // Note: [`UpdateFrequency::Slow`] means we have to hold down Space
            log::debug!("SPACE! We are going to SPACEEEEEEEE!");
        }
    }
}
