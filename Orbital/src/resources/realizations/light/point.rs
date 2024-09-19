use cgmath::{Vector3, Vector4, Zero};

#[derive(Debug)]
pub struct PointLight {
    pub label: String,
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl PointLight {
    pub fn dummy() -> Self {
        Self {
            label: "dummy".into(),
            position: Vector3::zero(),
            color: Vector3::zero(),
        }
    }

    pub fn bytes_needed() -> usize {
        // Note: Using Vector4 instead of Vector3 as in the struct to account
        // for padding
        // Position
        std::mem::size_of::<Vector4<f32>>() +
        // Color
        std::mem::size_of::<Vector4<f32>>()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.position.x.to_le_bytes(),
            self.position.y.to_le_bytes(),
            self.position.z.to_le_bytes(),
            [0u8; 4], // Padding
            self.color.x.to_le_bytes(),
            self.color.y.to_le_bytes(),
            self.color.z.to_le_bytes(),
            [0u8; 4], // Padding
        ]
        .concat()
        .to_vec()
    }
}
