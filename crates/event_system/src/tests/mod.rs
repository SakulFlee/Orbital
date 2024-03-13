use std::any::Any;

use entity_system::{entities, Entity};
use ulid::Ulid;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_test::wasm_bindgen_test;

use crate::{events, Event};
use console_log::console_log;

struct EntityTest {
    ulid: Ulid,
    success: bool,
}

impl EntityTest {
    pub fn new() -> Self {
        Self {
            ulid: Ulid::new(),
            success: false,
        }
    }
}

impl Entity for EntityTest {
    fn ulid(&self) -> &Ulid {
        &self.ulid
    }

    fn set_ulid(&mut self, ulid: Ulid) {
        self.ulid = ulid;
    }

    fn event_received(&mut self, _identifier: String, _event: &dyn Any) {
        println!("Event received!");
        self.success = true;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct EventTest;

impl Event for EventTest {
    fn identifier(&self) -> String {
        String::from("test")
    }
}

struct OtherEventTest;

impl Event for OtherEventTest {
    fn identifier(&self) -> String {
        String::from("other.test")
    }
}

#[wasm_bindgen_test]
fn event_dispatch() {
    spawn_local(async {
        // Make entity
        let entity = EntityTest::new();
        let ulid = *entity.ulid();
        entities()
            .lock()
            .expect("Mutex failure")
            .spawn(Box::new(entity))
            .expect("Spawn failure");

        // Register entity event listener
        events()
            .lock()
            .expect("Mutex failure")
            .register_receiver("test".into(), &ulid);

        // Dispatch event
        let event = EventTest {};
        events()
            .lock()
            .expect("Mutex failure")
            .dispatch_event(Box::new(event));

        // Poll
        events().lock().expect("Mutex failure").poll().await;

        // Check entity
        let entities = entities().lock().expect("Mutex failure");
        let entity = entities
            .get(&ulid)
            .expect("Spawn failure")
            .as_any()
            .downcast_ref::<EntityTest>()
            .expect("Any failure");

        assert!(entity.success);
    });
}

#[wasm_bindgen_test]
fn event_dispatch_with_wrong_identifier() {
    spawn_local(async {
        // Make entity
        let entity = EntityTest::new();
        let ulid = *entity.ulid();
        entities()
            .lock()
            .expect("Mutex failure")
            .spawn(Box::new(entity))
            .expect("Spawn failure");

        // Register entity event listener
        events()
            .lock()
            .expect("Mutex failure")
            .register_receiver("test".into(), &ulid);

        // Dispatch event
        let event = OtherEventTest {};
        events()
            .lock()
            .expect("Mutex failure")
            .dispatch_event(Box::new(event));

        // Poll
        events().lock().expect("Mutex failure").poll().await;

        // Check entity
        let entities = entities().lock().expect("Mutex failure");
        let entity = entities
            .get(&ulid)
            .expect("Spawn failure")
            .as_any()
            .downcast_ref::<EntityTest>()
            .expect("Any failure");

        assert!(!entity.success);
    });
}
