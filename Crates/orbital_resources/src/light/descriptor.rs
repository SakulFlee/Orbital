use cgmath::{Vector3, Zero};

#[derive(Debug, Clone, PartialEq)]
pub enum LightType {
    Point {
        intensity: f32,
    },
    Directional {
        intensity: f32,
    },
    Spot {
        intensity: f32,
        inner_cone_angle: f32,
        outer_cone_angle: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightDescriptor {
    pub label: String,
    pub light_type: LightType,
    pub color: Vector3<f32>,
    pub position: Vector3<f32>,
    pub direction: Vector3<f32>,
}

impl LightDescriptor {
    pub fn new_point(
        label: String,
        position: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
    ) -> Self {
        Self {
            label,
            light_type: LightType::Point { intensity },
            color,
            position,
            direction: Vector3::zero(),
        }
    }

    pub fn new_directional(
        label: String,
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
    ) -> Self {
        Self {
            label,
            light_type: LightType::Directional { intensity },
            color,
            position: Vector3::zero(),
            direction,
        }
    }

    pub fn new_spot(
        label: String,
        position: Vector3<f32>,
        direction: Vector3<f32>,
        color: Vector3<f32>,
        intensity: f32,
        inner_cone_angle: f32,
        outer_cone_angle: f32,
    ) -> Self {
        Self {
            label,
            light_type: LightType::Spot {
                intensity,
                inner_cone_angle,
                outer_cone_angle,
            },
            color,
            position,
            direction,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn to_buffer_data(&self) -> [u8; 64] {
        let mut data = [0u8; 64];
        let mut offset = 0;

        // Position (vec4) - 16 bytes
        // xyz: position, w: padding
        data[offset..offset + 4].copy_from_slice(&self.position.x.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&self.position.y.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&self.position.z.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&0f32.to_le_bytes()); // Padding
        offset += 4;

        // Color (vec4) - 16 bytes
        // xyz: color, w: intensity
        data[offset..offset + 4].copy_from_slice(&self.color.x.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&self.color.y.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&self.color.z.to_le_bytes());
        offset += 4;
        let intensity = match &self.light_type {
            LightType::Point { intensity } => *intensity,
            LightType::Directional { intensity } => *intensity,
            LightType::Spot { intensity, .. } => *intensity,
        };
        data[offset..offset + 4].copy_from_slice(&intensity.to_le_bytes());
        offset += 4;

        // Direction (vec4) - 16 bytes
        // xyz: direction, w: type
        data[offset..offset + 4].copy_from_slice(&self.direction.x.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&self.direction.y.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&self.direction.z.to_le_bytes());
        offset += 4;
        let light_type_value = match &self.light_type {
            LightType::Point { .. } => 0.0f32,       // LIGHT_TYPE_POINT
            LightType::Directional { .. } => 1.0f32, // LIGHT_TYPE_DIRECTIONAL
            LightType::Spot { .. } => 2.0f32,        // LIGHT_TYPE_SPOT
        };
        data[offset..offset + 4].copy_from_slice(&light_type_value.to_le_bytes());
        offset += 4;

        // Params (vec4) - 16 bytes
        // x: inner cone angle, y: outer cone angle, zw: padding
        let (inner_cone, outer_cone) = match &self.light_type {
            LightType::Point { .. } | LightType::Directional { .. } => (0.0f32, 0.0f32),
            LightType::Spot {
                inner_cone_angle,
                outer_cone_angle,
                ..
            } => (*inner_cone_angle, *outer_cone_angle),
        };
        data[offset..offset + 4].copy_from_slice(&inner_cone.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&outer_cone.to_le_bytes());
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&0f32.to_le_bytes()); // Padding
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&0f32.to_le_bytes()); // Padding

        data
    }
}

impl Default for LightDescriptor {
    fn default() -> Self {
        Self {
            label: "Default Light".to_string(),
            light_type: LightType::Point { intensity: 1.0 },
            color: Vector3::new(1.0, 1.0, 1.0),
            position: Vector3::zero(),
            direction: Vector3::new(0.0, -1.0, 0.0),
        }
    }
}
