use cgmath::{Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn plane(size: Vector2<f32>, subdivisions: u32) -> MeshDescriptor {
    let sw = subdivisions.max(1);
    let res = sw + 1;
    let mut vertices = Vec::with_capacity((res * res) as usize);
    let mut indices = Vec::with_capacity((6 * sw * sw) as usize);

    for j in 0..res {
        let z = -size.y / 2.0 + (j as f32 / sw as f32) * size.y;
        let v = 1.0 - j as f32 / sw as f32;
        for i in 0..res {
            let x = -size.x / 2.0 + (i as f32 / sw as f32) * size.x;
            let u = i as f32 / sw as f32;
            vertices.push(Vertex::new(
                Vector3::new(x, 0.0, z),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector2::new(u, v),
            ));
        }
    }

    for j in 0..sw {
        for i in 0..sw {
            let tl = (j * res + i);
            let tr = (j * res + i + 1);
            let bl = ((j + 1) * res + i);
            let br = ((j + 1) * res + i + 1);
            indices.extend_from_slice(&[tl, br, bl, tl, tr, br]);
        }
    }

    MeshDescriptor::new(vertices, indices)
}
