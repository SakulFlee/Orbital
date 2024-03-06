use cgmath::{Quaternion, Vector3};

use super::TInstance;

#[derive(Debug)]
pub struct StandardInstance {
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl TInstance for StandardInstance {
    fn new(position: Vector3<f32>, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }

    fn position(&self) -> Vector3<f32> {
        self.position
    }

    fn set_position(&mut self, postion: Vector3<f32>) {
        self.position = postion;
    }

    fn rotation(&self) -> Quaternion<f32> {
        self.rotation
    }

    fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rotation = rotation;
    }
}
