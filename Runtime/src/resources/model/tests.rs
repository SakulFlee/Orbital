use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use cgmath::{Vector2, Vector3};
use wgpu::TextureFormat;

use crate::{
    cache::Cache,
    resources::{MaterialDescriptor, MeshDescriptor, Transform, Vertex},
    wgpu_test_adapter,
};

use super::{Model, ModelDescriptor};

#[test]
fn realization() {
    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = ModelDescriptor {
        label: "Test".to_string(),
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
        transforms: vec![Transform::default()],
    };

    let cache_mesh = RwLock::new(Cache::new(Duration::from_secs(5)));
    let cache_material = RwLock::new(Cache::new(Duration::from_secs(5)));

    let _realization = Model::from_descriptor(
        &descriptor,
        &TextureFormat::Rgba8Uint,
        &device,
        &queue,
        &cache_mesh,
        &cache_material,
    )
    .expect("Failure realizing test model");
}
