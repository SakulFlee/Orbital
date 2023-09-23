use crate::app::{InputHandler, World};

use super::{EntityAction, EntityConfiguration};

pub trait TEntity {
    fn get_entity_config(&self) -> EntityConfiguration;

    fn update(&mut self, _delta_time: f64, _input_handler: &InputHandler) -> EntityAction {
        EntityAction::Keep
    }

    fn render(&self) {}
}
