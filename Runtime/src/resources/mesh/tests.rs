use cgmath::{Point3, Vector2, Vector3};

use crate::{
    resources::{Mesh, MeshDescriptor, Vertex},
    wgpu_test_adapter,
};

#[test]
fn realization() {
    let (_, device, queue) = wgpu_test_adapter::make_wgpu_connection();

    let descriptor = MeshDescriptor {
        vertices: vec![Vertex {
            position: Vector3::new(1.0, 2.0, 3.0),
            normal: Vector3::new(1.0, 2.0, 3.0),
            tangent: Vector3::new(1.0, 2.0, 3.0),
            bitangent: Vector3::new(1.0, 2.0, 3.0),
            uv: Vector2::new(1.0, 2.0),
        }],
        indices: vec![0],
    };

    let _realization = Mesh::from_descriptor(&descriptor, &device, &queue);
}

#[test]
fn bounding_box() {
    let descriptor = MeshDescriptor {
        vertices: vec![
            Vertex {
                position: Vector3::new(-5.0, -5.0, -5.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tangent: Vector3::new(0.0, 0.0, 0.0),
                bitangent: Vector3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(5.0, 5.0, 5.0),
                normal: Vector3::new(0.0, 0.0, 0.0),
                tangent: Vector3::new(0.0, 0.0, 0.0),
                bitangent: Vector3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(0.0, 0.0),
            },
        ],
        indices: vec![0],
    };

    let bounding_box = descriptor.find_bounding_box();

    assert_eq!(bounding_box.min, Point3::new(-5.0, -5.0, -5.0));
    assert_eq!(bounding_box.max, Point3::new(5.0, 5.0, 5.0));
}
