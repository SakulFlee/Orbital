use cgmath::{Point3, Vector2, Vector3};

use crate::resources::{BoundingBoxDescriptor, Mesh, MeshDescriptor, Vertex};

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
        bounding_box: BoundingBoxDescriptor {
            min: Point3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            max: Point3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        },
    };

    let _realization = Mesh::from_descriptor(&descriptor, &device, &queue);
}
