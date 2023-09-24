use crate::app::{
    EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency, World,
};

pub struct ClearScreenEntity {}

impl ClearScreenEntity {}

impl TEntity for ClearScreenEntity {
    fn get_entity_config(&self) -> EntityConfiguration {
        EntityConfiguration::new("Clear Screen Entity", UpdateFrequency::Slow, false)
    }

    fn update(&mut self, _delta_time: f64, _input_handler: &InputHandler) -> EntityAction {
        EntityAction::ClearColorAdjustment(World::SKY_BLUE_ISH_COLOR)
    }
}
