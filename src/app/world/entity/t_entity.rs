use crate::{
    app::InputHandler,
    engine::{EngineResult, LogicalDevice, TMesh},
};

use super::{EntityAction, EntityConfiguration};

pub trait TEntity {
    fn get_entity_configuration(&self) -> EntityConfiguration;

    fn update(&mut self, _delta_time: f64, _input_handler: &InputHandler) -> Vec<EntityAction> {
        vec![EntityAction::Keep]
    }

    fn prepare_render(&mut self, _logical_device: &LogicalDevice) -> EngineResult<()> {
        Ok(())
    }

    fn get_meshes(&self) -> Vec<&dyn TMesh> {
        vec![]
    }
}
