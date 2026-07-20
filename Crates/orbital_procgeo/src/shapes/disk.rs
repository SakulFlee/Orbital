use cgmath::{Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn disk(radius: f32, segments: u32) -> MeshDescriptor {
    let segs = segments.max(3);

    let mut vertices = Vec::with_capacity((segs + 2) as usize);
    let mut indices = Vec::with_capacity((3 * segs) as usize);

    // Center vertex
    vertices.push(Vertex::new(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector2::new(0.5, 0.5),
    ));

    // Ring vertices (segs+1 to close the loop)
    for i in 0..=segs {
        let theta = std::f32::consts::TAU * i as f32 / segs as f32;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let u = 0.5 + 0.5 * cos_t;
        let v = 0.5 + 0.5 * sin_t;

        vertices.push(Vertex::new(
            Vector3::new(radius * cos_t, 0.0, radius * sin_t),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(-sin_t, 0.0, cos_t),
            Vector2::new(u, v),
        ));
    }

    for i in 0..segs {
        indices.extend_from_slice(&[0, i + 1, i + 2]);
    }

    MeshDescriptor::new(vertices, indices)
}
