use cgmath::{Vector2, Vector3, InnerSpace};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn cylinder(radius: f32, height: f32, segments: u32) -> MeshDescriptor {
    let segs = segments.max(3);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Body vertices: top and bottom rings (segs+1 each to close UVs)
    for j in 0..=segs {
        let theta = std::f32::consts::TAU * j as f32 / segs as f32;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let u = j as f32 / segs as f32;

        let pos_bottom = Vector3::new(radius * cos_t, -height / 2.0, radius * sin_t);
        let pos_top = Vector3::new(radius * cos_t, height / 2.0, radius * sin_t);
        let normal = Vector3::new(cos_t, 0.0, sin_t);
        let tangent = Vector3::new(-sin_t, 0.0, cos_t).normalize();

        vertices.push(Vertex::new(pos_bottom, normal, tangent, Vector2::new(u, 0.0)));
        vertices.push(Vertex::new(pos_top, normal, tangent, Vector2::new(u, 1.0)));
    }

    // Body indices (quad strip)
    for i in 0..segs {
        let b0 = (i * 2) as u32;
        let t0 = (i * 2 + 1) as u32;
        let b1 = ((i + 1) * 2) as u32;
        let t1 = ((i + 1) * 2 + 1) as u32;
        indices.extend_from_slice(&[b0, t1, b1, b0, t0, t1]);
    }

    // Top cap
    let top_center_idx = vertices.len() as u32;
    vertices.push(Vertex::new(
        Vector3::new(0.0, height / 2.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector2::new(0.5, 0.5),
    ));
    let top_start = vertices.len() as u32;
    for j in 0..=segs {
        let theta = std::f32::consts::TAU * j as f32 / segs as f32;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let u = 0.5 + 0.5 * cos_t;
        let v = 0.5 + 0.5 * sin_t;
        vertices.push(Vertex::new(
            Vector3::new(radius * cos_t, height / 2.0, radius * sin_t),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector2::new(u, v),
        ));
    }
    for i in 0..segs {
        indices.extend_from_slice(&[top_center_idx, top_start + i + 1, top_start + i]);
    }

    // Bottom cap
    let bot_center_idx = vertices.len() as u32;
    vertices.push(Vertex::new(
        Vector3::new(0.0, -height / 2.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector2::new(0.5, 0.5),
    ));
    let bot_start = vertices.len() as u32;
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
        indices.extend_from_slice(&[bot_center_idx, bot_start + i, bot_start + i + 1]);
    }

    MeshDescriptor::new(vertices, indices)
}
