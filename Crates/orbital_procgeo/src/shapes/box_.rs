use cgmath::{Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

struct Face {
    origin: Vector3<f32>,
    u_axis: Vector3<f32>,
    v_axis: Vector3<f32>,
    normal: Vector3<f32>,
    tangent: Vector3<f32>,
    size_u: f32,
    size_v: f32,
}

fn face_vertices(face: &Face) -> [Vertex; 4] {
    let p0 = face.origin;
    let p1 = face.origin + face.u_axis * face.size_u;
    let p2 = face.origin + face.u_axis * face.size_u + face.v_axis * face.size_v;
    let p3 = face.origin + face.v_axis * face.size_v;

    [
        Vertex::new(p0, face.normal, face.tangent, Vector2::new(0.0, 0.0)),
        Vertex::new(p1, face.normal, face.tangent, Vector2::new(1.0, 0.0)),
        Vertex::new(p2, face.normal, face.tangent, Vector2::new(1.0, 1.0)),
        Vertex::new(p3, face.normal, face.tangent, Vector2::new(0.0, 1.0)),
    ]
}

fn face_indices_left(base: u32) -> [u32; 6] {
    [base, base + 1, base + 2, base, base + 2, base + 3]
}

fn face_indices_right(base: u32) -> [u32; 6] {
    [base, base + 2, base + 1, base, base + 3, base + 2]
}

pub fn box_(size: Vector3<f32>) -> MeshDescriptor {
    let w = size.x / 2.0;
    let h = size.y / 2.0;
    let d = size.z / 2.0;

    let faces = [
        // +Y (top)
        Face {
            origin: Vector3::new(-w, h, -d),
            u_axis: Vector3::new(1.0, 0.0, 0.0),
            v_axis: Vector3::new(0.0, 0.0, 1.0),
            normal: Vector3::new(0.0, 1.0, 0.0),
            tangent: Vector3::new(1.0, 0.0, 0.0),
            size_u: size.x,
            size_v: size.z,
        },
        // -Y (bottom)
        Face {
            origin: Vector3::new(-w, -h, d),
            u_axis: Vector3::new(1.0, 0.0, 0.0),
            v_axis: Vector3::new(0.0, 0.0, -1.0),
            normal: Vector3::new(0.0, -1.0, 0.0),
            tangent: Vector3::new(1.0, 0.0, 0.0),
            size_u: size.x,
            size_v: size.z,
        },
        // +X (right)
        Face {
            origin: Vector3::new(w, -h, -d),
            u_axis: Vector3::new(0.0, 0.0, 1.0),
            v_axis: Vector3::new(0.0, 1.0, 0.0),
            normal: Vector3::new(1.0, 0.0, 0.0),
            tangent: Vector3::new(0.0, 0.0, 1.0),
            size_u: size.z,
            size_v: size.y,
        },
        // -X (left)
        Face {
            origin: Vector3::new(-w, -h, d),
            u_axis: Vector3::new(0.0, 0.0, -1.0),
            v_axis: Vector3::new(0.0, 1.0, 0.0),
            normal: Vector3::new(-1.0, 0.0, 0.0),
            tangent: Vector3::new(0.0, 0.0, -1.0),
            size_u: size.z,
            size_v: size.y,
        },
        // +Z (front)
        Face {
            origin: Vector3::new(-w, -h, d),
            u_axis: Vector3::new(1.0, 0.0, 0.0),
            v_axis: Vector3::new(0.0, 1.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 1.0),
            tangent: Vector3::new(1.0, 0.0, 0.0),
            size_u: size.x,
            size_v: size.y,
        },
        // -Z (back)
        Face {
            origin: Vector3::new(w, -h, -d),
            u_axis: Vector3::new(-1.0, 0.0, 0.0),
            v_axis: Vector3::new(0.0, 1.0, 0.0),
            normal: Vector3::new(0.0, 0.0, -1.0),
            tangent: Vector3::new(-1.0, 0.0, 0.0),
            size_u: size.x,
            size_v: size.y,
        },
    ];

    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (fi, face) in faces.iter().enumerate() {
        let base = (fi * 4) as u32;
        vertices.extend_from_slice(&face_vertices(face));
        if fi == 4 || fi == 5 {
            indices.extend_from_slice(&face_indices_right(base));
        } else {
            indices.extend_from_slice(&face_indices_left(base));
        }
    }

    MeshDescriptor::new(vertices, indices)
}
