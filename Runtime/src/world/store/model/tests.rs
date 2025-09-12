use std::sync::Arc;

use cgmath::{Vector2, Vector3};
use hashbrown::HashMap;
use ulid::Ulid;

use crate::{
    element::ModelEvent,
    resources::{MaterialDescriptor, MeshDescriptor, ModelDescriptor, Transform, Vertex},
    world::store::model::ModelStore,
};

#[test]
fn test_basic_instancing() {
    let mut store = ModelStore::new();

    // Create a base model descriptor
    let mut transforms = HashMap::new();
    transforms.insert(Ulid::new(), Transform::default());

    let base_descriptor = ModelDescriptor {
        label: "Cube".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms,
    };

    // Spawn the base model
    store.handle_event(ModelEvent::Spawn(base_descriptor.clone()));

    // Verify base model was stored
    assert!(store.label_to_id("Cube").is_some());
    let base_id = store.label_to_id("Cube").unwrap();
    assert_eq!(store.map_descriptors[&base_id].transforms.len(), 1);

    // Spawn a duplicate model (same mesh and materials, different label)
    let mut duplicate_transforms = HashMap::new();
    duplicate_transforms.insert(
        Ulid::new(),
        Transform {
            position: Vector3::new(5.0, 0.0, 0.0),
            ..Default::default()
        },
    );

    let duplicate_descriptor = ModelDescriptor {
        label: "Cube2".to_string(),
        mesh: base_descriptor.mesh.clone(),
        materials: base_descriptor.materials.clone(),
        transforms: duplicate_transforms,
    };

    store.handle_event(ModelEvent::Spawn(duplicate_descriptor));

    // Verify instancing occurred
    assert!(store.label_to_id("Cube2").is_some());
    let instance_id = store.label_to_id("Cube2").unwrap();

    // The instance should point to the same base model
    assert_eq!(base_id, instance_id);

    // The base model should now have 2 transforms
    assert_eq!(store.map_descriptors[&base_id].transforms.len(), 2);

    // Verify instance tracking
    assert!(store.instance_tracker.contains_key("Cube2"));
    let (tracked_base_label, _) = &store.instance_tracker["Cube2"];
    assert_eq!(tracked_base_label, "Cube");
}

#[test]
fn test_instance_despawning() {
    let mut store = ModelStore::new();

    // Create and spawn base model
    let mut transforms = HashMap::new();
    let base_ulid = Ulid::new();
    transforms.insert(base_ulid, Transform::default());

    let base_descriptor = ModelDescriptor {
        label: "BaseCube".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms,
    };

    store.handle_event(ModelEvent::Spawn(base_descriptor.clone()));

    // Create and spawn instance
    let mut instance_transforms = HashMap::new();
    instance_transforms.insert(
        Ulid::new(),
        Transform {
            position: Vector3::new(10.0, 0.0, 0.0),
            ..Default::default()
        },
    );

    let instance_descriptor = ModelDescriptor {
        label: "InstanceCube".to_string(),
        mesh: base_descriptor.mesh.clone(),
        materials: base_descriptor.materials.clone(),
        transforms: instance_transforms,
    };

    store.handle_event(ModelEvent::Spawn(instance_descriptor));

    let base_id = store.label_to_id("BaseCube").unwrap();
    assert_eq!(store.map_descriptors[&base_id].transforms.len(), 2);

    // Despawn the instance
    store.handle_event(ModelEvent::Despawn("InstanceCube".to_string()));

    // Verify instance was removed
    assert!(!store.instance_tracker.contains_key("InstanceCube"));
    assert!(!store.map_label.contains_key("InstanceCube"));

    // Verify base model still has only 1 transform
    assert_eq!(store.map_descriptors[&base_id].transforms.len(), 1);

    // Verify the remaining transform is the original one
    let remaining_transform = store.map_descriptors[&base_id]
        .transforms
        .values()
        .next()
        .unwrap();
    assert_eq!(remaining_transform.position, Vector3::new(0.0, 0.0, 0.0));
}

#[test]
fn test_base_model_despawning_with_instances() {
    let mut store = ModelStore::new();

    // Create and spawn base model
    let mut transforms = HashMap::new();
    transforms.insert(Ulid::new(), Transform::default());

    let base_descriptor = ModelDescriptor {
        label: "BaseCube".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms,
    };

    store.handle_event(ModelEvent::Spawn(base_descriptor.clone()));

    // Create multiple instances
    for i in 1..=3 {
        let mut instance_transforms = HashMap::new();
        instance_transforms.insert(
            Ulid::new(),
            Transform {
                position: Vector3::new(i as f32 * 5.0, 0.0, 0.0),
                ..Default::default()
            },
        );

        let instance_descriptor = ModelDescriptor {
            label: format!("InstanceCube{}", i),
            mesh: base_descriptor.mesh.clone(),
            materials: base_descriptor.materials.clone(),
            transforms: instance_transforms,
        };

        store.handle_event(ModelEvent::Spawn(instance_descriptor));
    }

    let base_id = store.label_to_id("BaseCube").unwrap();
    assert_eq!(store.map_descriptors[&base_id].transforms.len(), 4); // 1 base + 3 instances

    // Despawn the base model
    store.handle_event(ModelEvent::Despawn("BaseCube".to_string()));

    // Verify everything was cleaned up
    assert!(!store.map_label.contains_key("BaseCube"));
    assert!(!store.map_descriptors.contains_key(&base_id));
    assert!(!store.instance_tracker.contains_key("InstanceCube1"));
    assert!(!store.instance_tracker.contains_key("InstanceCube2"));
    assert!(!store.instance_tracker.contains_key("InstanceCube3"));
    assert!(!store.map_label.contains_key("InstanceCube1"));
    assert!(!store.map_label.contains_key("InstanceCube2"));
    assert!(!store.map_label.contains_key("InstanceCube3"));
}

#[test]
fn test_different_materials_prevent_instancing() {
    let mut store = ModelStore::new();

    // Create first model
    let mut transforms1 = HashMap::new();
    transforms1.insert(Ulid::new(), Transform::default());

    let descriptor1 = ModelDescriptor {
        label: "Cube1".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms: transforms1,
    };

    store.handle_event(ModelEvent::Spawn(descriptor1.clone()));

    // Create second model with different material
    let mut different_material = MaterialDescriptor::default();
    different_material.name = Some("Different".to_string());

    let mut transforms2 = HashMap::new();
    transforms2.insert(Ulid::new(), Transform::default());

    let descriptor2 = ModelDescriptor {
        label: "Cube2".to_string(),
        mesh: descriptor1.mesh.clone(),                // Same mesh
        materials: vec![Arc::new(different_material)], // Different material
        transforms: transforms2,
    };

    store.handle_event(ModelEvent::Spawn(descriptor2));

    // Verify they are NOT instanced (different materials)
    let id1 = store.label_to_id("Cube1").unwrap();
    let id2 = store.label_to_id("Cube2").unwrap();
    assert_ne!(id1, id2); // Different IDs

    // Each should have only 1 transform
    assert_eq!(store.map_descriptors[&id1].transforms.len(), 1);
    assert_eq!(store.map_descriptors[&id2].transforms.len(), 1);
}

#[test]
fn test_different_meshes_prevent_instancing() {
    let mut store = ModelStore::new();

    // Create first model
    let mut transforms1 = HashMap::new();
    transforms1.insert(Ulid::new(), Transform::default());

    let descriptor1 = ModelDescriptor {
        label: "Cube1".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms: transforms1,
    };

    store.handle_event(ModelEvent::Spawn(descriptor1.clone()));

    // Create second model with different mesh
    let mut transforms2 = HashMap::new();
    transforms2.insert(Ulid::new(), Transform::default());

    let descriptor2 = ModelDescriptor {
        label: "Cube2".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(2.0, 3.0, 4.0), // Different vertex
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: descriptor1.materials.clone(), // Same material
        transforms: transforms2,
    };

    store.handle_event(ModelEvent::Spawn(descriptor2));

    // Verify they are NOT instanced (different meshes)
    let id1 = store.label_to_id("Cube1").unwrap();
    let id2 = store.label_to_id("Cube2").unwrap();
    assert_ne!(id1, id2); // Different IDs

    // Each should have only 1 transform
    assert_eq!(store.map_descriptors[&id1].transforms.len(), 1);
    assert_eq!(store.map_descriptors[&id2].transforms.len(), 1);
}

#[test]
fn test_instance_hash_consistency() {
    let mut transforms = HashMap::new();
    transforms.insert(Ulid::new(), Transform::default());

    let descriptor1 = ModelDescriptor {
        label: "Cube1".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms: transforms.clone(),
    };

    let descriptor2 = ModelDescriptor {
        label: "Cube2".to_string(),
        mesh: descriptor1.mesh.clone(),
        materials: descriptor1.materials.clone(),
        transforms: transforms.clone(),
    };

    // Hash should be the same for identical mesh/material combinations
    assert_eq!(descriptor1.instance_hash(), descriptor2.instance_hash());
}

#[test]
fn test_instance_label_generation() {
    let mut store = ModelStore::new();

    // Create base model
    let mut transforms = HashMap::new();
    transforms.insert(Ulid::new(), Transform::default());

    let base_descriptor = ModelDescriptor {
        label: "Base".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms,
    };

    store.handle_event(ModelEvent::Spawn(base_descriptor.clone()));

    // Create instance
    let mut instance_transforms = HashMap::new();
    instance_transforms.insert(Ulid::new(), Transform::default());

    let instance_descriptor = ModelDescriptor {
        label: "Instance".to_string(),
        mesh: base_descriptor.mesh.clone(),
        materials: base_descriptor.materials.clone(),
        transforms: instance_transforms,
    };

    store.handle_event(ModelEvent::Spawn(instance_descriptor));

    // Check that an instance label was generated
    let instance_labels: Vec<&String> = store.instance_tracker.keys().collect();
    assert_eq!(instance_labels.len(), 1);
    assert_eq!(instance_labels[0], "Instance"); // Should use the original label since it doesn't conflict
}

#[test]
fn test_clear_cleans_up_instancing_data() {
    let mut store = ModelStore::new();

    // Create base model
    let mut transforms = HashMap::new();
    transforms.insert(Ulid::new(), Transform::default());

    let base_descriptor = ModelDescriptor {
        label: "Base".to_string(),
        mesh: Arc::new(MeshDescriptor {
            vertices: vec![Vertex {
                position: Vector3::new(1.0, 2.0, 3.0),
                normal: Vector3::new(1.0, 2.0, 3.0),
                tangent: Vector3::new(1.0, 2.0, 3.0),
                bitangent: Vector3::new(1.0, 2.0, 3.0),
                uv: Vector2::new(1.0, 2.0),
            }],
            indices: vec![0],
        }),
        materials: vec![Arc::new(MaterialDescriptor::default())],
        transforms,
    };

    store.handle_event(ModelEvent::Spawn(base_descriptor.clone()));

    // Create instance
    let mut instance_transforms = HashMap::new();
    instance_transforms.insert(Ulid::new(), Transform::default());

    let instance_descriptor = ModelDescriptor {
        label: "Instance".to_string(),
        mesh: base_descriptor.mesh.clone(),
        materials: base_descriptor.materials.clone(),
        transforms: instance_transforms,
    };

    store.handle_event(ModelEvent::Spawn(instance_descriptor));

    // Verify data exists
    assert!(!store.instance_map.is_empty());
    assert!(!store.instance_tracker.is_empty());

    // Clear the store
    store.clear().unwrap();

    // Verify instancing data was cleared
    assert!(store.instance_map.is_empty());
    assert!(store.instance_tracker.is_empty());
    assert!(store.map_descriptors.is_empty());
    assert!(store.map_label.is_empty());
}
