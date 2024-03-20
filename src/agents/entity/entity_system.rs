use std::sync::{Mutex, OnceLock};

use hashbrown::HashMap;
use ulid::Ulid;

use super::BoxedEntity;

#[derive(Default)]
pub struct EntitySystem {
    entities: HashMap<Ulid, BoxedEntity>,
}
pub type Entities = EntitySystem;

impl EntitySystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, entity: BoxedEntity) -> Result<(), String> {
        if self.entities.contains_key(entity.ulid()) {
            return Err(format!(
                "Entity with ULID '{}' already exists!",
                entity.ulid()
            ));
        }

        self.entities.insert(*entity.ulid(), entity);
        Ok(())
    }

    pub fn despawn(&mut self, ulid: &Ulid) -> Result<BoxedEntity, String> {
        match self.entities.remove(ulid) {
            Some(e) => Ok(e),
            None => Err(format!("Entity with ULID '{}' doesn't exist!", ulid)),
        }
    }

    pub fn contains(&self, ulid: &Ulid) -> bool {
        self.entities.contains_key(ulid)
    }

    pub fn get(&self, ulid: &Ulid) -> Result<&BoxedEntity, String> {
        match self.entities.get(ulid) {
            Some(e) => Ok(e),
            None => Err(format!("Entity with ULID '{}' doesn't exist!", ulid)),
        }
    }

    pub fn get_mut(&mut self, ulid: &Ulid) -> Result<&mut BoxedEntity, String> {
        match self.entities.get_mut(ulid) {
            Some(e) => Ok(e),
            None => Err(format!("Entity with ULID '{}' doesn't exist!", ulid)),
        }
    }
}

pub fn entity_system() -> &'static Mutex<EntitySystem> {
    static ENTITY_SYSTEM: OnceLock<Mutex<EntitySystem>> = OnceLock::new();
    &ENTITY_SYSTEM.get_or_init(|| Mutex::new(EntitySystem::new()))
}

pub fn entities() -> &'static Mutex<EntitySystem> {
    entity_system()
}
