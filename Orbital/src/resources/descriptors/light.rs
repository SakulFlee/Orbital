use cgmath::Vector3;

#[derive(Debug)]
pub enum LightDescriptor {
    PointLight {
        position: Vector3<f32>,
        color: Vector3<f32>,
    },
}

impl LightDescriptor {
    pub fn bytes_needed(&self) -> u64 {
        match self {
            LightDescriptor::PointLight { position, color } => {
                // + 4 for padding
                ((std::mem::size_of_val(position) + 4) + (std::mem::size_of_val(color) + 4)) as u64
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            LightDescriptor::PointLight { position, color } => [
                position.x.to_le_bytes(),
                position.y.to_le_bytes(),
                position.z.to_le_bytes(),
                [0u8; 4],
                color.x.to_le_bytes(),
                color.y.to_le_bytes(),
                color.z.to_le_bytes(),
                [0u8; 4],
            ]
            .concat()
            .to_vec(),
        }
    }
}
