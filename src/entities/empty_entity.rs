use crate::app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency};

pub struct EmptyEntity {
    tag: String,
}

impl EmptyEntity {
    pub fn new<S>(tag: S) -> Self
    where
        S: Into<String>,
    {
        Self { tag: tag.into() }
    }
}

impl TEntity for EmptyEntity {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new(self.tag.clone(), UpdateFrequency::Slow, false)
    }

    fn update(&mut self, delta_time: f64, _input_handler: &InputHandler) -> Vec<EntityAction> {
        log::debug!("I am an empty entity! (delta: {delta_time}ms)");
        vec![EntityAction::Keep]
    }
}
