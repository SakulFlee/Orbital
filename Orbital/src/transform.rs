use cgmath::{Quaternion, Vector3, Zero};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn only_position(position: Vector3<f32>) -> Self {
        Self {
            position,
            rotation: Quaternion::zero(),
            scale: Vector3::zero(),
        }
    }

    pub fn only_rotation(rotation: Quaternion<f32>) -> Self {
        Self {
            position: Vector3::zero(),
            rotation,
            scale: Vector3::zero(),
        }
    }

    pub fn only_scale(scale: Vector3<f32>) -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale,
        }
    }

    pub fn zero() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::zero(),
        }
    }

    pub fn apply_transform(&mut self, transform: Transform) {
        self.apply_position(transform.position);
        self.apply_rotation(transform.rotation);
        self.apply_scale(transform.scale);
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn set_position(&mut self, new_position: Vector3<f32>) {
        self.position = new_position;
    }

    pub fn apply_position(&mut self, other_position: Vector3<f32>) {
        self.position += other_position;
    }

    pub fn rotation(&self) -> Quaternion<f32> {
        self.rotation
    }

    pub fn set_rotation(&mut self, new_rotation: Quaternion<f32>) {
        self.rotation = new_rotation;
    }

    pub fn apply_rotation(&mut self, other_rotation: Quaternion<f32>) {
        self.rotation += other_rotation;
    }

    pub fn scale(&self) -> Vector3<f32> {
        self.scale
    }

    pub fn set_scale(&mut self, new_scale: Vector3<f32>) {
        self.scale = new_scale;
    }

    pub fn apply_scale(&mut self, other_scale: Vector3<f32>) {
        self.scale += other_scale;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
            rotation: Quaternion::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}
