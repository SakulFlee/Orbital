use crate::resources::realizations::PointLight;

#[derive(Debug)]
pub enum LightDescriptor {
    PointLight(PointLight),
}

impl LightDescriptor {
    pub fn label(&self) -> &str {
        match self {
            LightDescriptor::PointLight(point_light) => &point_light.label,
        }
    }

    pub fn bytes_needed(&self) -> u64 {
        match self {
            LightDescriptor::PointLight(point_light) => {
                // + 1 for padding. Could also use Vec4 instead of 3.
                ((std::mem::size_of_val(&point_light.position) + 1)
                    + (std::mem::size_of_val(&point_light.color) + 1)) as u64
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            LightDescriptor::PointLight(point_light) => [
                point_light.position.x.to_le_bytes(),
                point_light.position.y.to_le_bytes(),
                point_light.position.z.to_le_bytes(),
                [0u8; 4],
                point_light.color.x.to_le_bytes(),
                point_light.color.y.to_le_bytes(),
                point_light.color.z.to_le_bytes(),
                [0u8; 4],
            ]
            .concat()
            .to_vec(),
        }
    }
}
