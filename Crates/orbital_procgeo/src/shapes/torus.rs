use cgmath::{InnerSpace, Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn torus(
    major_radius: f32,
    minor_radius: f32,
    major_segments: u32,
    minor_segments: u32,
) -> MeshDescriptor {
    let maj = major_segments.max(3);
    let min = minor_segments.max(3);

    let mut vertices = Vec::with_capacity(((maj + 1) * (min + 1)) as usize);
    let mut indices = Vec::with_capacity((6 * maj * min) as usize);

    for j in 0..=maj {
        let theta = std::f32::consts::TAU * j as f32 / maj as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let u = j as f32 / maj as f32;

        for i in 0..=min {
            let phi = std::f32::consts::TAU * i as f32 / min as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            let v = i as f32 / min as f32;

            let pos = Vector3::new(
                (major_radius + minor_radius * cos_phi) * cos_theta,
                minor_radius * sin_phi,
                (major_radius + minor_radius * cos_phi) * sin_theta,
            );

            let center_to_point = Vector3::new(
                minor_radius * cos_phi * cos_theta,
                minor_radius * sin_phi,
                minor_radius * cos_phi * sin_theta,
            );
            let normal = center_to_point.normalize();

            let tangent = Vector3::new(-sin_theta, 0.0, cos_theta).normalize();

            vertices.push(Vertex::new(pos, normal, tangent, Vector2::new(u, v)));
        }
    }

    let row = min + 1;
    for j in 0..maj {
        for i in 0..min {
            let tl = (j * row + i);
            let tr = (j * row + i + 1);
            let bl = ((j + 1) * row + i);
            let br = ((j + 1) * row + i + 1);
            indices.extend_from_slice(&[tl, bl, br, tl, br, tr]);
        }
    }

    MeshDescriptor::new(vertices, indices)
}
