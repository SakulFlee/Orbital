use crate::{
    entity::{entities, Entity},
    event::events,
};

mod entity_test;
use entity_test::*;

mod event_test;
use event_test::*;

mod event_test_other;
use event_test_other::*;

mod event_counter_test;
use event_counter_test::*;

#[test]
fn event_dispatch() {
    pollster::block_on(async {
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

        assert!(entity.success());
    });
}

#[test]
fn event_dispatch_with_wrong_identifier() {
    pollster::block_on(async {
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

        assert!(!entity.success());
    });
}

#[test]
fn event_dispatch_count() {
    pollster::block_on(async {
        // Make entity
        let entity = EntityCountTest::new();
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
            .register_receiver("counter.test".into(), &ulid);

        // Dispatch event
        let event = EventTestCounter {};
        events()
            .lock()
            .expect("Mutex failure")
            .dispatch_event(Box::new(event));

        // Poll a few times
        let mut events_lock = events().lock().expect("Mutex failure");
        for _ in 0..=3 {
            events_lock.poll().await;
        }

        // Check entity
        let entities = entities().lock().expect("Mutex failure");
        let entity = entities
            .get(&ulid)
            .expect("Spawn failure")
            .as_any()
            .downcast_ref::<EntityCountTest>()
            .expect("Any failure");

        // If the event would still have been in queue during our multiple
        // polls above, this counter would be more than 1.
        // If polling didn't work for some reason, the counter should be 0.
        // If the counter is 1, it means the event was processed exactly once.
        assert_eq!(entity.counter(), 1);
    });
}
