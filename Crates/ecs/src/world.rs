use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use crate::{ComponentStore, Entity, EntityIdType, WorldComponentStorage};

#[derive(Debug)]
pub struct World {
    pub entity_counter: EntityIdType,
    pub entity_ids: Vec<Entity>,
    // TODO: Recycle entity IDs
    pub component_stores: HashMap<TypeId, Box<dyn WorldComponentStorage>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_counter: 0,
            entity_ids: Vec::new(),
            component_stores: HashMap::new(),
        }
    }

    pub fn spawn_entity(&mut self) -> Entity {
        let entity_id = self.entity_counter;
        self.entity_counter = self.entity_counter.wrapping_add(1);

        let entity = Entity::new(entity_id);
        self.entity_ids.push(entity);

        entity
    }

    pub fn despawn_entity(&mut self, entity: &Entity) {
        self.entity_ids.retain(|e| e.index != entity.index);
        self.component_stores
            .iter_mut()
            .for_each(|(_, x)| x.remove_entity(entity.index));
    }

    pub fn get_component_store<T: Any + Debug>(&self) -> Option<&ComponentStore<T>> {
        let type_id = TypeId::of::<T>();

        self.component_stores
            .get(&type_id)
            .and_then(|store| (**store).as_any().downcast_ref::<ComponentStore<T>>())
    }
}
