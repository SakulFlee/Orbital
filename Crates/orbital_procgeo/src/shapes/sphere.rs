use cgmath::{InnerSpace, Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn uv_sphere(radius: f32, segments: u32, rings: u32) -> MeshDescriptor {
    let segs = segments.max(3);
    let rngs = rings.max(2);

    let mut vertices = Vec::with_capacity(((rngs + 1) * (segs + 1)) as usize);
    let mut indices = Vec::with_capacity((6 * rngs * segs) as usize);

    for j in 0..=rngs {
        let phi = std::f32::consts::PI * j as f32 / rngs as f32;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let v = j as f32 / rngs as f32;

        for i in 0..=segs {
            let theta = std::f32::consts::TAU * i as f32 / segs as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let u = i as f32 / segs as f32;

            let pos = Vector3::new(
                radius * sin_phi * cos_theta,
                radius * cos_phi,
                radius * sin_phi * sin_theta,
            );
            let normal = pos.normalize();

            let tangent = Vector3::new(-sin_theta, 0.0, cos_theta).normalize();

            vertices.push(Vertex::new(pos, normal, tangent, Vector2::new(u, v)));
        }
    }

    let row = segs + 1;
    for j in 0..rngs {
        for i in 0..segs {
            let tl = j * row + i;
            let tr = j * row + i + 1;
            let bl = (j + 1) * row + i;
            let br = (j + 1) * row + i + 1;
            indices.extend_from_slice(&[tl, bl, br, tl, br, tr]);
        }
    }

    MeshDescriptor::new(vertices, indices)
}
