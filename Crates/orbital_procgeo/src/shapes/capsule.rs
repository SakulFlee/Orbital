use cgmath::{InnerSpace, Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> MeshDescriptor {
    let segs = segments.max(3);
    let rngs = rings.max(4);
    let rngs_per_hemi = rngs / 2;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate profile from bottom pole (index 0) to top pole (index num_rows-1)
    // Row 0: bottom pole (y = -height/2 - radius)
    // Rows 1..rngs_per_hemi: bottom hemisphere (y from -height/2 - radius to -height/2)
    // Row rngs_per_hemi: bottom hemisphere equator (y = -height/2)
    // Rows rngs_per_hemi..(rngs - rngs_per_hemi): cylinder body
    // Row (rngs - rngs_per_hemi): top hemisphere equator (y = height/2)
    // Rows (rngs - rngs_per_hemi + 1)..rngs: top hemisphere
    // Row rngs: top pole (y = height/2 + radius)

    let num_rows = rngs + 1;
    let row_size = segs + 1;

    for row in 0..num_rows {
        let (y, normal_y, sin_horiz, _cos_horiz) = if row <= rngs_per_hemi {
            // Bottom hemisphere: angle from -π/2 at pole to 0 at equator
            let t = row as f32 / rngs_per_hemi.max(1) as f32;
            let angle = -std::f32::consts::FRAC_PI_2 + t * std::f32::consts::FRAC_PI_2;
            let y = -height / 2.0 + radius * angle.sin();
            (y, angle.sin(), angle.cos(), 1.0f32.max(0.0))
        } else if row >= rngs - rngs_per_hemi {
            // Top hemisphere: angle from 0 at equator to π/2 at pole
            let t = (row - (rngs - rngs_per_hemi)) as f32 / rngs_per_hemi.max(1) as f32;
            let angle = t * std::f32::consts::FRAC_PI_2;
            let y = height / 2.0 + radius * angle.sin();
            (y, angle.sin(), angle.cos(), 1.0f32.max(0.0))
        } else {
            // Cylinder body
            let t = (row - rngs_per_hemi) as f32 / (rngs - 2 * rngs_per_hemi).max(1) as f32;
            let y = -height / 2.0 + t * height;
            (y, 0.0, 1.0, 0.0)
        };

        let horiz_radius = radius * sin_horiz.max(0.0);

        for col in 0..row_size {
            let theta = std::f32::consts::TAU * col as f32 / segs as f32;
            let sin_t = theta.sin();
            let cos_t = theta.cos();
            let u = col as f32 / segs as f32;
            let v = row as f32 / rngs as f32;

            let pos = Vector3::new(horiz_radius * cos_t, y, horiz_radius * sin_t);
            let normal = Vector3::new(sin_horiz * cos_t, normal_y, sin_horiz * sin_t);
            let normal = if normal.magnitude2() > 0.0 {
                normal.normalize()
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            let tangent = Vector3::new(-sin_t, 0.0, cos_t).normalize();

            vertices.push(Vertex::new(pos, normal, tangent, Vector2::new(u, v)));
        }
    }

    for row in 0..rngs {
        for col in 0..segs {
            let tl = (row * row_size + col) as u32;
            let tr = (row * row_size + col + 1) as u32;
            let bl = ((row + 1) * row_size + col) as u32;
            let br = ((row + 1) * row_size + col + 1) as u32;
            indices.extend_from_slice(&[tl, br, bl, tl, tr, br]);
        }
    }

    MeshDescriptor::new(vertices, indices)
}
