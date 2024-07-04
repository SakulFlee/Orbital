use cgmath::{Quaternion, Vector3, Zero};

#[derive(Debug, Clone)]
pub enum Instancing {
    Single(InstanceDescriptor),
    Multiple(Vec<InstanceDescriptor>),
}

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
