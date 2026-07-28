use cgmath::{InnerSpace, Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn cone(radius: f32, height: f32, segments: u32) -> MeshDescriptor {
    let segs = segments.max(3);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Body: triangle strip from base edge ring to apex
    let apex_idx = vertices.len() as u32;
    vertices.push(Vertex::new(
        Vector3::new(0.0, height / 2.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector2::new(0.5, 1.0),
    ));

    let base_start = vertices.len() as u32;
    for j in 0..=segs {
        let theta = std::f32::consts::TAU * j as f32 / segs as f32;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let u = j as f32 / segs as f32;

        let pos = Vector3::new(radius * cos_t, -height / 2.0, radius * sin_t);
        let to_apex = Vector3::new(0.0, height / 2.0, 0.0) - pos;
        let edge_dir = Vector3::new(-sin_t, 0.0, cos_t);
        let normal = to_apex.cross(edge_dir).normalize();
        let tangent = edge_dir.normalize();

        vertices.push(Vertex::new(pos, normal, tangent, Vector2::new(u, 0.0)));
    }

    for i in 0..segs {
        indices.extend_from_slice(&[apex_idx, base_start + i, base_start + i + 1]);
    }

    // Base cap (triangle fan)
    let bot_center_idx = vertices.len() as u32;
    vertices.push(Vertex::new(
        Vector3::new(0.0, -height / 2.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector2::new(0.5, 0.5),
    ));

    let bot_ring_start = vertices.len() as u32;
    for j in 0..=segs {
        let theta = std::f32::consts::TAU * j as f32 / segs as f32;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let u = 0.5 + 0.5 * cos_t;
        let v = 0.5 + 0.5 * sin_t;
        vertices.push(Vertex::new(
            Vector3::new(radius * cos_t, -height / 2.0, radius * sin_t),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector2::new(u, v),
        ));
    }

    for i in 0..segs {
        indices.extend_from_slice(&[bot_center_idx, bot_ring_start + i, bot_ring_start + i + 1]);
    }

    MeshDescriptor::new(vertices, indices)
}
