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

    pub fn to_buffer_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Position (vec4) - 16 bytes
        // xyz: position, w: padding
        data.extend_from_slice(&self.position.x.to_le_bytes());
        data.extend_from_slice(&self.position.y.to_le_bytes());
        data.extend_from_slice(&self.position.z.to_le_bytes());
        data.extend_from_slice(&0f32.to_le_bytes()); // Padding

        // Color (vec4) - 16 bytes
        // xyz: color, w: intensity
        data.extend_from_slice(&self.color.x.to_le_bytes());
        data.extend_from_slice(&self.color.y.to_le_bytes());
        data.extend_from_slice(&self.color.z.to_le_bytes());
        let intensity = match &self.light_type {
            LightType::Point { intensity } => *intensity,
            LightType::Directional { intensity } => *intensity,
            LightType::Spot { intensity, .. } => *intensity,
        };
        data.extend_from_slice(&intensity.to_le_bytes()); // Intensity

        // Direction (vec4) - 16 bytes
        // xyz: direction, w: type
        data.extend_from_slice(&self.direction.x.to_le_bytes());
        data.extend_from_slice(&self.direction.y.to_le_bytes());
        data.extend_from_slice(&self.direction.z.to_le_bytes());
        let light_type_value = match &self.light_type {
            LightType::Point { .. } => 0.0f32,       // LIGHT_TYPE_POINT
            LightType::Directional { .. } => 1.0f32, // LIGHT_TYPE_DIRECTIONAL
            LightType::Spot { .. } => 2.0f32,        // LIGHT_TYPE_SPOT
        };
        data.extend_from_slice(&light_type_value.to_le_bytes()); // Light type

        // Params (vec4) - 16 bytes
        // Spot lights: x/y = angular attenuation scale/offset (see below), zw: padding.
        // Other lights: all zeros.
        match &self.light_type {
            LightType::Spot {
                inner_cone_angle,
                outer_cone_angle,
                ..
            } => {
                let (scale, offset) = spot_angular_attenuation(*inner_cone_angle, *outer_cone_angle);
                data.extend_from_slice(&scale.to_le_bytes()); // Attenuation scale
                data.extend_from_slice(&offset.to_le_bytes()); // Attenuation offset
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
            }
            _ => {
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
                data.extend_from_slice(&0f32.to_le_bytes()); // Padding
            }
        }

        data
    }
}

/// Precompute the glTF `KHR_lights_punctual` angular attenuation coefficients
/// for a spot light, in the cosine domain:
///
/// ```text
/// angular_attenuation = clamp(cos_theta * scale + offset, 0, 1)
/// ```
///
/// where `cos_theta` is the cosine of the angle between the light direction
/// and the direction from the light to the shaded point. The denominator is
/// clamped so `inner == outer` (a hard-edged cone) cannot divide by zero.
fn spot_angular_attenuation(inner_cone_angle: f32, outer_cone_angle: f32) -> (f32, f32) {
    // glTF requires inner < outer; tolerate swapped/reversed input.
    let inner = inner_cone_angle.min(outer_cone_angle).max(0.0);
    let outer = inner_cone_angle.max(outer_cone_angle).max(0.0);
    let cos_inner = inner.cos();
    let cos_outer = outer.cos();
    let scale = 1.0 / (cos_inner - cos_outer).max(1e-4);
    let offset = -cos_outer * scale;
    (scale, offset)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spot_attenuation_plateau_and_edges() {
        let inner = 0.3f32;
        let outer = 0.5f32;
        let (scale, offset) = spot_angular_attenuation(inner, outer);

        // Plateau: everything inside the inner cone clamps to 1
        assert!(scale + offset >= 1.0, "cone center must be fully lit");
        assert!(
            inner.cos() * scale + offset >= 1.0 - 1e-4,
            "inner cone edge must be (almost) fully lit"
        );
        // Outer edge falls to 0
        assert!(
            (outer.cos() * scale + offset).abs() < 1e-4,
            "outer cone edge must be fully dark"
        );
    }

    #[test]
    fn spot_attenuation_equal_angles_stays_finite() {
        let (scale, offset) = spot_angular_attenuation(0.4, 0.4);
        assert!(scale.is_finite() && offset.is_finite());
    }

    #[test]
    fn spot_attenuation_swapped_angles_are_sorted() {
        assert_eq!(
            spot_angular_attenuation(0.5, 0.3),
            spot_angular_attenuation(0.3, 0.5)
        );
    }

    #[test]
    fn spot_buffer_packs_scale_offset_and_type() {
        let light = LightDescriptor::new_spot(
            "s".to_string(),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            10.0,
            0.3,
            0.5,
        );
        let data = light.to_buffer_data();
        assert_eq!(data.len(), 64, "one light must stay 64 bytes");

        let intensity = f32::from_le_bytes(data[28..32].try_into().unwrap());
        assert_eq!(intensity, 10.0);
        let type_id = f32::from_le_bytes(data[44..48].try_into().unwrap());
        assert_eq!(type_id, 2.0);

        let scale = f32::from_le_bytes(data[48..52].try_into().unwrap());
        let offset = f32::from_le_bytes(data[52..56].try_into().unwrap());
        let (s, o) = spot_angular_attenuation(0.3, 0.5);
        assert_eq!((scale, offset), (s, o));
    }
}
