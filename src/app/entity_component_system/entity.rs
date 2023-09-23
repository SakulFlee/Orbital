use std::collections::HashMap;

use crate::engine::{EngineError, EngineResult};

use super::Component;

pub struct Entity {
    components: HashMap<String, Box<dyn Component>>,
}

impl Entity {
    pub fn empty() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn from_components(components: Vec<(String, Box<dyn Component>)>) -> Self {
        let mut entity = Entity::empty();

        for (tag, component) in components {
            entity.add_component(tag, component);
        }

        entity
    }

    fn has_component<S>(&self, tag: S) -> bool
    where
        S: Into<String>,
    {
        self.components.contains_key(&tag.into())
    }

    fn add_component<S>(&mut self, tag: S, component: Box<dyn Component>)
    where
        S: Into<String>,
    {
        self.components.insert(tag.into(), component);
    }

    fn get_component<S>(&self, tag: S) -> EngineResult<&Box<dyn Component>>
    where
        S: Into<String>,
    {
        self.components
            .get(&tag.into())
            .map(|x| Ok(x))
            .unwrap_or(Err(EngineError::ComponentTagMissing))
    }

    fn get_component_mut<S>(&mut self, tag: S) -> EngineResult<&mut Box<dyn Component>>
    where
        S: Into<String>,
    {
        self.components
            .get_mut(&tag.into())
            .map(|x| Ok(x))
            .unwrap_or(Err(EngineError::ComponentTagMissing))
    }

    fn get_and_cast_component<T>(&mut self, tag: &str) -> EngineResult<&T>
    where
        T: Component + 'static,
    {
        self.get_component(tag)?
            .as_any()
            .downcast_ref::<T>()
            .map_or(Err(EngineError::ComponentCastFailure), |x| Ok(x))
    }

    fn get_and_cast_component_mut<T>(&mut self, tag: &str) -> EngineResult<&mut T>
    where
        T: Component + 'static,
    {
        self.get_component_mut(tag)?
            .as_any_mut()
            .downcast_mut::<T>()
            .map_or(Err(EngineError::ComponentCastFailure), |x| Ok(x))
    }
}
