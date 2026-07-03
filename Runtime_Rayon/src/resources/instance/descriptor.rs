use super::Transform;
use cgmath::{Quaternion, Vector3, Zero};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InstanceDescriptor {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for InstanceDescriptor {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl From<&Transform> for InstanceDescriptor {
    fn from(value: &Transform) -> Self {
        Self {
            position: value.position,
            rotation: value.rotation,
            scale: value.scale,
        }
    }
}
