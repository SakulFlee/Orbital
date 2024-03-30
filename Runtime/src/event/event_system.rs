use hashbrown::HashMap;
use log::debug;
use std::{
    any::Any,
    sync::{Mutex, OnceLock},
};
use ulid::Ulid;

use crate::entity::entity_system;

use super::BoxedEvent;

#[derive(Default)]
pub struct EventSystem {
    events: Vec<BoxedEvent>,
    receivers: HashMap<String, Vec<Ulid>>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn poll(&mut self) {
        let mut entity_system = entity_system().lock().expect("Mutex failure");

        for event in self.events.drain(..) {
            debug!("Dispatching event: {}", event.identifier());

            if let Some(entities) = self.receivers.get(&event.identifier()) {
                for entity in entities {
                    let e = entity_system.get_mut(entity).expect("Entity doesn't exist");

                    debug!("Sending event to entity: {}", e.ulid());

                    let any: &dyn Any = &event;
                    e.event_received(event.identifier(), any);
                }
            }
        }
    }

    pub fn dispatch_event(&mut self, event: BoxedEvent) {
        self.events.push(event);
    }

    pub fn register_receiver(&mut self, identifier: String, entity_id: &Ulid) {
        if !self.receivers.contains_key(&identifier) {
            let mut v: Vec<Ulid> = Vec::new();
            v.push(*entity_id);

            self.receivers.insert(identifier, v);
        } else {
            let v = self
                .receivers
                .get_mut(&identifier)
                .expect("key is contained, this should not fail!");

            v.push(*entity_id);
        }
    }
}

pub fn event_system() -> &'static Mutex<EventSystem> {
    static EVENT_SYSTEM: OnceLock<Mutex<EventSystem>> = OnceLock::new();
    EVENT_SYSTEM.get_or_init(|| Mutex::new(EventSystem::new()))
}

pub fn events() -> &'static Mutex<EventSystem> {
    event_system()
}
