use crate::app::InputHandler;

use super::EntityConfiguration;

pub trait TEntity {
    fn get_entity_config(&self) -> EntityConfiguration;

    fn update(&mut self, delta_time: f64, input_handler: &InputHandler) {}

    fn render(&self) {}
}
