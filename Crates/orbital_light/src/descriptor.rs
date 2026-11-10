use cgmath::{Vector3, Zero};

/// Default radius for manually created point/spot lights, in world units.
///
/// glTF `KHR_lights_punctual` lights use their explicit `range` when present.
pub const DEFAULT_LIGHT_RANGE: f32 = 10.0;

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
    /// Point/spot influence radius in world units. Ignored for directional lights.
    pub range: f32,
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
            range: DEFAULT_LIGHT_RANGE,
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
            range: DEFAULT_LIGHT_RANGE,
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
            range: DEFAULT_LIGHT_RANGE,
        }
    }

    /// Sets the point/spot influence radius in world units.
    pub fn with_range(mut self, range: f32) -> Self {
        self.range = range.max(0.0);
        self
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
        // x/y = spot angular attenuation scale/offset (0 for non-spot lights)
        // z = range² for distance culling (0 for directional = infinite range)
        // w = padding
        let range_sq = match &self.light_type {
            LightType::Directional { .. } => 0.0f32,
            LightType::Point { .. } | LightType::Spot { .. } => {
                let range = if self.range.is_finite() && self.range > 0.0 {
                    self.range
                } else {
                    DEFAULT_LIGHT_RANGE
                };
                range * range
            }
        };
        let (attn_scale, attn_offset) = match &self.light_type {
            LightType::Directional { .. } | LightType::Point { .. } => (0.0f32, 0.0f32),
            LightType::Spot {
                inner_cone_angle,
                outer_cone_angle,
                ..
            } => spot_angular_attenuation(*inner_cone_angle, *outer_cone_angle),
        };
        data[offset..offset + 4].copy_from_slice(&attn_scale.to_le_bytes()); // Attenuation scale
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&attn_offset.to_le_bytes()); // Attenuation offset
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&range_sq.to_le_bytes()); // Range²
        offset += 4;
        data[offset..offset + 4].copy_from_slice(&0f32.to_le_bytes()); // Padding

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
            range: DEFAULT_LIGHT_RANGE,
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

        // range² stored in params.z for distance culling
        let range_sq = f32::from_le_bytes(data[56..60].try_into().unwrap());
        assert_eq!(range_sq, DEFAULT_LIGHT_RANGE * DEFAULT_LIGHT_RANGE);
    }

    #[test]
    fn point_buffer_packs_type_and_range_without_attenuation() {
        let light = LightDescriptor::new_point(
            "p".to_string(),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            10.0,
        );
        let data = light.to_buffer_data();
        assert_eq!(data.len(), 64, "one light must stay 64 bytes");

        let type_id = f32::from_le_bytes(data[44..48].try_into().unwrap());
        assert_eq!(type_id, 0.0, "point light");

        // Point lights have no cone, so params.x/y must stay zeroed
        let scale = f32::from_le_bytes(data[48..52].try_into().unwrap());
        let offset = f32::from_le_bytes(data[52..56].try_into().unwrap());
        assert_eq!((scale, offset), (0.0, 0.0));

        let range_sq = f32::from_le_bytes(data[56..60].try_into().unwrap());
        assert_eq!(range_sq, DEFAULT_LIGHT_RANGE * DEFAULT_LIGHT_RANGE);

        // params.w padding
        assert_eq!(data[60..64], [0u8; 4]);
    }

    #[test]
    fn point_buffer_packs_explicit_range() {
        let light = LightDescriptor::new_point(
            "p".to_string(),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            10.0,
        )
        .with_range(25.0);
        let data = light.to_buffer_data();
        let range_sq = f32::from_le_bytes(data[56..60].try_into().unwrap());
        assert_eq!(range_sq, 625.0);
    }

    #[test]
    fn zero_range_falls_back_to_default() {
        let light = LightDescriptor::new_point(
            "p".to_string(),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            10.0,
        )
        .with_range(0.0);
        let data = light.to_buffer_data();
        let range_sq = f32::from_le_bytes(data[56..60].try_into().unwrap());
        assert_eq!(range_sq, DEFAULT_LIGHT_RANGE * DEFAULT_LIGHT_RANGE);
    }

    #[test]
    fn directional_buffer_has_infinite_range() {
        let light = LightDescriptor::new_directional(
            "d".to_string(),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            10.0,
        );
        let data = light.to_buffer_data();
        assert_eq!(data.len(), 64, "one light must stay 64 bytes");

        let type_id = f32::from_le_bytes(data[44..48].try_into().unwrap());
        assert_eq!(type_id, 1.0, "directional light");

        let scale = f32::from_le_bytes(data[48..52].try_into().unwrap());
        let offset = f32::from_le_bytes(data[52..56].try_into().unwrap());
        assert_eq!((scale, offset), (0.0, 0.0));

        // range² = 0 tells the shader the light is never distance-culled
        let range_sq = f32::from_le_bytes(data[56..60].try_into().unwrap());
        assert_eq!(range_sq, 0.0);
    }
}
