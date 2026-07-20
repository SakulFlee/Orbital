use std::collections::HashMap;

use cgmath::{Vector2, Vector3, InnerSpace};
use orbital_resources::{MeshDescriptor, Vertex};

const PHI: f32 = 1.618033988749895;

fn icosahedron_vertices() -> [Vector3<f32>; 12] {
    [
        Vector3::new(0.0, -1.0, -PHI),
        Vector3::new(0.0, -1.0, PHI),
        Vector3::new(0.0, 1.0, -PHI),
        Vector3::new(0.0, 1.0, PHI),
        Vector3::new(-1.0, -PHI, 0.0),
        Vector3::new(-1.0, PHI, 0.0),
        Vector3::new(1.0, -PHI, 0.0),
        Vector3::new(1.0, PHI, 0.0),
        Vector3::new(-PHI, 0.0, -1.0),
        Vector3::new(-PHI, 0.0, 1.0),
        Vector3::new(PHI, 0.0, -1.0),
        Vector3::new(PHI, 0.0, 1.0),
    ]
}

fn icosahedron_faces() -> [[usize; 3]; 20] {
    [
        [0, 2, 8],
        [0, 2, 10],
        [0, 4, 6],
        [0, 4, 8],
        [0, 6, 10],
        [1, 3, 9],
        [1, 3, 11],
        [1, 4, 6],
        [1, 4, 9],
        [1, 6, 11],
        [2, 5, 7],
        [2, 5, 8],
        [2, 7, 10],
        [3, 5, 7],
        [3, 5, 9],
        [3, 7, 11],
        [4, 8, 9],
        [5, 8, 9],
        [6, 10, 11],
        [7, 10, 11],
    ]
}

fn midpoint_index(
    cache: &mut HashMap<(usize, usize), usize>,
    vertices: &mut Vec<Vector3<f32>>,
    a: usize,
    b: usize,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let mid = (vertices[a] + vertices[b]) / 2.0;
    let len = mid.magnitude();
    let normalized = if len > 0.0 { mid / len } else { mid };
    let idx = vertices.len();
    vertices.push(normalized);
    cache.insert(key, idx);
    idx
}

pub fn icosphere(radius: f32, subdivisions: u32) -> MeshDescriptor {
    let mut verts: Vec<Vector3<f32>> = icosahedron_vertices()
        .iter()
        .map(|v| {
            let len = v.magnitude();
            if len > 0.0 { *v / len } else { *v }
        })
        .collect();

    let mut faces: Vec<[usize; 3]> = icosahedron_faces().to_vec();

    for _ in 0..subdivisions {
        let mut cache = HashMap::new();
        let mut new_faces = Vec::with_capacity(faces.len() * 4);

        for &[v0, v1, v2] in &faces {
            let a = midpoint_index(&mut cache, &mut verts, v0, v1);
            let b = midpoint_index(&mut cache, &mut verts, v1, v2);
            let c = midpoint_index(&mut cache, &mut verts, v2, v0);

            new_faces.push([v0, a, c]);
            new_faces.push([v1, b, a]);
            new_faces.push([v2, c, b]);
            new_faces.push([a, b, c]);
        }

        faces = new_faces;
    }

    let mut vertices = Vec::with_capacity(verts.len());
    let mut indices = Vec::with_capacity(faces.len() * 3);

    for &v in &verts {
        let pos = v * radius;
        let normal = pos.normalize();
        let tangent = {
            let t = normal.cross(Vector3::unit_y());
            if t.magnitude2() < 0.0001 {
                normal.cross(Vector3::unit_x()).normalize()
            } else {
                t.normalize()
            }
        };

        let theta = f32::atan2(pos.z, pos.x);
        let phi = (pos.y / radius).acos();
        let u = 0.5 + theta / std::f32::consts::TAU;
        let v_uv = 1.0 - phi / std::f32::consts::PI;
        vertices.push(Vertex::new(pos, normal, tangent, Vector2::new(u, v_uv)));
    }

    for &[v0, v1, v2] in &faces {
        let p0 = verts[v0];
        let p1 = verts[v1];
        let p2 = verts[v2];
        let e1 = p1 - p0;
        let e2 = p2 - p1;
        let cross = e1.cross(e2);
        // cross pointing outward (same direction as vertex) → CCW → flip
        if cross.dot(p0) > 0.0 {
            indices.extend_from_slice(&[v0 as u32, v2 as u32, v1 as u32]);
        } else {
            indices.extend_from_slice(&[v0 as u32, v1 as u32, v2 as u32]);
        }
    }

    MeshDescriptor::new(vertices, indices)
}
