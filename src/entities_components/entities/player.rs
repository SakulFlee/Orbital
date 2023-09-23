use crate::app::{component_value, Entity};

pub struct Player(Entity);

impl Player {
    pub fn new() -> Self {
        Self(Entity::from_components(vec![component_value!(Health, 100)]))
    }
}
