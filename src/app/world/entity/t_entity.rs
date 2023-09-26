use wgpu::{Device, Queue};

use crate::{
    app::InputHandler,
    engine::{EngineResult, TMesh},
};

use super::{EntityAction, EntityConfiguration};

pub trait TEntity {
    fn get_entity_configuration(&self) -> EntityConfiguration;

    // TODO: Vec<EntityAction>
    fn update(&mut self, _delta_time: f64, _input_handler: &InputHandler) -> EntityAction {
        EntityAction::Keep
    }

    fn prepare_render(&mut self, _device: &Device, _queue: &Queue) -> EngineResult<()> {
        Ok(())
    }

    fn get_meshes(&self) -> Vec<&dyn TMesh> {
        vec![]
    }
}
