use std::time::Instant;

use cgmath::{Quaternion, Vector3};
use hashbrown::HashMap;
use loader::Loader;
use model::Transform;
use world_change::WorldChange;

use crate::{GLTFIdentifier, GLTFLoader, GLTFWorkerMode};

const TEST_FILE_PATH: &str = "../../Assets/Models/PBR_Spheres.glb";
const TEST_FILE_WORLD_CHANGES: usize = 121;

#[test]
fn load_everything() {
    let mut loader = GLTFLoader::new(TEST_FILE_PATH, GLTFWorkerMode::LoadEverything, None);

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
        None,
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
        None,
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
        None,
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

#[test]
fn test_transform_single() {
    let transform = Transform {
        position: Vector3::new(1.0, 2.0, 3.0),
        rotation: Quaternion::new(4.0, 5.0, 6.0, 7.0),
        scale: Vector3::new(8.0, 9.0, 10.0),
    };

    let mut models = HashMap::new();
    models.insert(
        GLTFIdentifier::Label("Scene"),
        GLTFIdentifier::ranged_id(0, 0),
    );

    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFWorkerMode::LoadSpecific {
            scene_model_map: Some(models),
            scene_light_map: None,
            scene_camera_map: None,
        },
        Some(transform),
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
    assert_eq!(world_changes.len(), 1);

    if let WorldChange::SpawnModel(model_descriptor) = world_changes.first().unwrap() {
        let model_transform = model_descriptor.transforms.first().unwrap();

        assert_eq!(model_transform.position.x, transform.position.x);
        assert_eq!(model_transform.position.y, transform.position.y);
        assert_eq!(model_transform.position.z, transform.position.z);

        assert_eq!(model_transform.rotation.s, transform.rotation.s);
        assert_eq!(model_transform.rotation.v, transform.rotation.v);

        assert_eq!(model_transform.scale.x, transform.scale.x);
        assert_eq!(model_transform.scale.y, transform.scale.y);
        assert_eq!(model_transform.scale.z, transform.scale.z);
    } else {
        panic!("Invalid model import");
    }
}

#[test]
fn test_transform_multiple_same() {
    let transform = Transform {
        position: Vector3::new(1.0, 2.0, 3.0),
        rotation: Quaternion::new(4.0, 5.0, 6.0, 7.0),
        scale: Vector3::new(8.0, 9.0, 10.0),
    };

    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFWorkerMode::LoadEverything,
        Some(transform),
    );

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    let world_changes = result.expect("Must be non-error");
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), TEST_FILE_WORLD_CHANGES);

    let mut i = 0;
    for world_change in world_changes {
        println!("#{}", i);
        i += 1;

        if let WorldChange::SpawnModel(model_descriptor) = world_change {
            println!(">>> {}", model_descriptor.transforms.len());
            let model_transform = model_descriptor.transforms.first().unwrap();

            println!(">>> >>> {:?}", model_transform);

            assert_eq!(model_transform.position.x, transform.position.x);
            assert_eq!(model_transform.position.y, transform.position.y);
            assert_eq!(model_transform.position.z, transform.position.z);

            assert_eq!(model_transform.rotation.s, transform.rotation.s);
            assert_eq!(model_transform.rotation.v, transform.rotation.v);

            assert_eq!(model_transform.scale.x, transform.scale.x);
            assert_eq!(model_transform.scale.y, transform.scale.y);
            assert_eq!(model_transform.scale.z, transform.scale.z);
        } else {
            panic!("Invalid model import");
        }
    }
}
