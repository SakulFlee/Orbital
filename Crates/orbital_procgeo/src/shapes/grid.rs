use cgmath::{Vector2, Vector3};
use orbital_resources::{MeshDescriptor, Vertex};

pub fn grid(width: f32, depth: f32, cols: u32, rows: u32) -> MeshDescriptor {
    let c = cols.max(1);
    let r = rows.max(1);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for j in 0..=r {
        let z = -depth / 2.0 + (j as f32 / r as f32) * depth;
        let v = 1.0 - j as f32 / r as f32;
        for i in 0..=c {
            let x = -width / 2.0 + (i as f32 / c as f32) * width;
            let u = i as f32 / c as f32;
            vertices.push(Vertex::new(
                Vector3::new(x, 0.0, z),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector2::new(u, v),
            ));
        }
    }

    let stride = c + 1;

    // Horizontal lines (along X, at each Z row)
    for j in 0..=r {
        for i in 0..c {
            let a = j * stride + i;
            let b = j * stride + i + 1;
            indices.push(a as u32);
            indices.push(b as u32);
        }
    }

    // Vertical lines (along Z, at each X column)
    for i in 0..=c {
        for j in 0..r {
            let a = j * stride + i;
            let b = (j + 1) * stride + i;
            indices.push(a as u32);
            indices.push(b as u32);
        }
    }

    MeshDescriptor::new(vertices, indices)
}
