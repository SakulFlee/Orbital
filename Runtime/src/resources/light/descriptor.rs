use cgmath::{Vector3, Zero};
use std::f32::consts::PI;

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
    pub fn new_point(label: String, position: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
        Self {
            label,
            light_type: LightType::Point { intensity },
            color,
            position,
            direction: Vector3::zero(),
        }
    }

    pub fn new_directional(label: String, direction: Vector3<f32>, color: Vector3<f32>, intensity: f32) -> Self {
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

    pub fn to_buffer_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Position (vec3) - 12 bytes
        data.extend_from_slice(&self.position.x.to_le_bytes());
        data.extend_from_slice(&self.position.y.to_le_bytes());
        data.extend_from_slice(&self.position.z.to_le_bytes());
        data.extend_from_slice(&0f32.to_le_bytes()); // Padding to 16 bytes
        
        // Color (vec3) - 12 bytes
        data.extend_from_slice(&self.color.x.to_le_bytes());
        data.extend_from_slice(&self.color.y.to_le_bytes());
        data.extend_from_slice(&self.color.z.to_le_bytes());
        data.extend_from_slice(&0f32.to_le_bytes()); // Padding to 16 bytes
        
        // Direction (vec3) - 12 bytes
        data.extend_from_slice(&self.direction.x.to_le_bytes());
        data.extend_from_slice(&self.direction.y.to_le_bytes());
        data.extend_from_slice(&self.direction.z.to_le_bytes());
        data.extend_from_slice(&0f32.to_le_bytes()); // Padding to 16 bytes
        
        // Light type specific data
        match &self.light_type {
            LightType::Point { intensity } => {
                data.extend_from_slice(&intensity.to_le_bytes());
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
            }
            LightType::Directional { intensity } => {
                data.extend_from_slice(&intensity.to_le_bytes());
                data.extend_from_slice(&1f32.to_le_bytes()); // Type identifier
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
            }
            LightType::Spot {
                intensity,
                inner_cone_angle,
                outer_cone_angle,
            } => {
                data.extend_from_slice(&intensity.to_le_bytes());
                data.extend_from_slice(&2f32.to_le_bytes()); // Type identifier
                data.extend_from_slice(&inner_cone_angle.to_le_bytes());
                data.extend_from_slice(&outer_cone_angle.to_le_bytes());
            }
        }
        
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
