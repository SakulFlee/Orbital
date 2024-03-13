use std::sync::{Mutex, OnceLock};

use crate::EventSystem;

pub fn event_system() -> &'static Mutex<EventSystem> {
    static EVENT_SYSTEM: OnceLock<Mutex<EventSystem>> = OnceLock::new();
    &EVENT_SYSTEM.get_or_init(|| Mutex::new(EventSystem::new()))
}

pub fn events() -> &'static Mutex<EventSystem> {
    event_system()
}
