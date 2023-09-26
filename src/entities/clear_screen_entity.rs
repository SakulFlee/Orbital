use crate::app::{
    EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency, World,
};

#[derive(Debug, Default)]
pub struct ClearScreenEntity {}

impl ClearScreenEntity {}

impl TEntity for ClearScreenEntity {
    fn get_entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new("Clear Screen Entity", UpdateFrequency::Slow, false)
    }

    fn update(&mut self, _delta_time: f64, _input_handler: &InputHandler) -> Vec<EntityAction> {
        vec![EntityAction::ClearColorAdjustment(
            World::SKY_BLUE_ISH_COLOR,
        )]
    }
}
