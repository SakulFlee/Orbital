use std::time::Instant;

use hashbrown::HashMap;

use crate::loader::{GLTFIdentifier, GLTFLoader, GLTFWorkerMode, Worker};

const TEST_FILE_PATH: &'static str = "../Assets/Models/PBR_Spheres.glb";
const TEST_FILE_WORLD_CHANGES: usize = 121;

#[test]
fn load_everything() {
    let mut loader = GLTFLoader::new(TEST_FILE_PATH, GLTFWorkerMode::LoadEverything);

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();
    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), TEST_FILE_WORLD_CHANGES);
}

#[test]
fn load_scene_id() {
    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFWorkerMode::LoadScenes {
            scene_identifiers: vec![
                GLTFIdentifier::Id(0),
                GLTFIdentifier::Id(1),
                GLTFIdentifier::Id(2),
            ],
        },
    );

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), TEST_FILE_WORLD_CHANGES);
}

#[test]
fn load_scene_label() {
    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFWorkerMode::LoadScenes {
            scene_identifiers: vec![GLTFIdentifier::Label("Scene")],
        },
    );

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), TEST_FILE_WORLD_CHANGES);
}

#[test]
fn load_scene_specific() {
    let mut models = HashMap::new();
    models.insert(
        GLTFIdentifier::Label("Scene"),
        GLTFIdentifier::ranged_id(0, 2),
    );

    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFWorkerMode::LoadSpecific {
            scene_model_map: Some(models),
            scene_light_map: None,
            scene_camera_map: None,
        },
    );

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), 3);
}
