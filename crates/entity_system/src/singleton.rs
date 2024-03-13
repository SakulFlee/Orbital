use std::sync::{Mutex, OnceLock};

use crate::EntitySystem;

pub fn entity_system() -> &'static Mutex<EntitySystem> {
    static ENTITY_SYSTEM: OnceLock<Mutex<EntitySystem>> = OnceLock::new();
    &ENTITY_SYSTEM.get_or_init(|| Mutex::new(EntitySystem::new()))
}

pub fn entities() -> &'static Mutex<EntitySystem> {
    entity_system()
}
