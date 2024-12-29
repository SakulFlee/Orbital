use std::{cmp::Ordering, f32, hash::Hash};

use cgmath::{
    num_traits::{Float, FloatConst},
    Point3, Vector3,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub a: Point3<f32>,
    pub b: Point3<f32>,
}

impl BoundingBox {
    pub fn to_binary_data(&self) -> Vec<u8> {
        vec![
            // A
            self.a.x.to_le_bytes(),
            self.a.y.to_le_bytes(),
            self.a.z.to_le_bytes(),
            // B
            self.b.x.to_le_bytes(),
            self.b.y.to_le_bytes(),
            self.b.z.to_le_bytes(),
            // Buffer alignment to 32b
            [0u8; 4],
            [0u8; 4],
        ]
        .concat()
    }

    pub fn to_binary_data_disabled_frustum_culling() -> Vec<u8> {
        vec![
            // A
            [0u8; 4], [0u8; 4], [0u8; 4], // B
            [0u8; 4], [0u8; 4], [0u8; 4],
        ]
        .concat()
    }
}

impl Eq for BoundingBox {}

impl Hash for BoundingBox {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.a.x.integer_decode().hash(state);
        self.a.y.integer_decode().hash(state);
        self.a.z.integer_decode().hash(state);
        self.b.x.integer_decode().hash(state);
        self.b.y.integer_decode().hash(state);
        self.b.z.integer_decode().hash(state);
    }
}
