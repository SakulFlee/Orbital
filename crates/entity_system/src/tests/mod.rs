use ulid::Ulid;
use wasm_bindgen_test::*;

use crate::{entities, entity_system, Entity};

struct EntityTest {
    ulid: Ulid,
}

impl EntityTest {
    pub fn new() -> Self {
        Self { ulid: Ulid::new() }
    }
}

impl Entity for EntityTest {
    fn ulid(&self) -> &Ulid {
        &self.ulid
    }

    fn set_ulid(&mut self, ulid: Ulid) {
        self.ulid = ulid;
    }
}

#[wasm_bindgen_test]
fn static_fn_entities() {
    // Check aliases
    let lock = entities().lock().expect("Mutex failure");
    drop(lock);
}

#[wasm_bindgen_test]
fn static_fn_entity_system() {
    // Check aliases
    let lock = entity_system().lock().expect("Mutex failure");
    drop(lock);
}

#[wasm_bindgen_test]
fn entity_spawn() {
    let entity = Box::new(EntityTest::new());

    // Spawn the entity
    entities()
        .lock()
        .expect("Mutex failure")
        .spawn(entity)
        .expect("Spawning failure");
}

#[wasm_bindgen_test]
fn entity_spawn_and_get() {
    let entity = EntityTest::new();
    let ulid_copy = *entity.ulid();
    let boxed_entity = Box::new(entity);

    // Spawn the entity
    entities()
        .lock()
        .expect("Mutex failure")
        .spawn(boxed_entity)
        .expect("Spawning failure");

    // Try retrieving the entity
    let entity_system = entities().lock().expect("Mutex failure");
    let retrieved_entity = entity_system.get(&ulid_copy).expect("Entity missing");

    assert_eq!(ulid_copy, *retrieved_entity.ulid())
}

#[wasm_bindgen_test]
fn entity_spawn_and_get_mut() {
    let entity = EntityTest::new();
    let ulid_copy = *entity.ulid();
    let boxed_entity = Box::new(entity);

    // Spawn the entity
    entities()
        .lock()
        .expect("Mutex failure")
        .spawn(boxed_entity)
        .expect("Spawning failure");

    // Try retrieving the entity
    let mut entity_system = entities().lock().expect("Mutex failure");
    let retrieved_entity = entity_system.get_mut(&ulid_copy).expect("Entity missing");

    assert_eq!(ulid_copy, *retrieved_entity.ulid())
}

#[wasm_bindgen_test]
fn entity_spawn_and_contains() {
    let entity = EntityTest::new();
    let ulid_copy = *entity.ulid();
    let boxed_entity = Box::new(entity);

    // Spawn the entity
    entities()
        .lock()
        .expect("Mutex failure")
        .spawn(boxed_entity)
        .expect("Spawning failure");

    // Check if the entity exists
    let exists = entities()
        .lock()
        .expect("Mutex failure")
        .contains(&ulid_copy);
    assert!(exists);
}

#[wasm_bindgen_test]
fn entity_spawn_and_despawn() {
    let entity = EntityTest::new();
    let ulid_copy = *entity.ulid();
    let boxed_entity = Box::new(entity);

    // Spawn the entity
    entities()
        .lock()
        .expect("Mutex failure")
        .spawn(boxed_entity)
        .expect("Spawning failure");

    // Check if the entity exists
    assert!(entities()
        .lock()
        .expect("Mutex failure")
        .contains(&ulid_copy));

    // Despawn entity and
    let despawned_entity = entities()
        .lock()
        .expect("Mutex failure")
        .despawn(&ulid_copy)
        .expect("Despawn failure");
    assert_eq!(*despawned_entity.ulid(), ulid_copy);

    // Check if it no longer exists
    assert!(!entities()
        .lock()
        .expect("Mutex failure")
        .contains(&ulid_copy));
}

#[wasm_bindgen_test]
fn check_despawn_invalid_id() {
    let ulid = Ulid::new();

    let result = entities().lock().expect("Mutex failure").despawn(&ulid);

    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn contains_invalid_id() {
    let ulid = Ulid::new();

    let result = entities().lock().expect("Mutex failure").contains(&ulid);

    assert!(!result);
}

#[wasm_bindgen_test]
fn get_invalid_id() {
    let ulid = Ulid::new();

    let entity_system = entities().lock().expect("Mutex failure");
    let result = entity_system.get(&ulid);

    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn get_mut_invalid_id() {
    let ulid = Ulid::new();

    let mut entity_system = entities().lock().expect("Mutex failure");
    let result = entity_system.get_mut(&ulid);

    assert!(result.is_err());
}
