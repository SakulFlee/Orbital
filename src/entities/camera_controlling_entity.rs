use cgmath::{Point3, Vector3};
use winit::event::VirtualKeyCode;

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::{Camera, CameraChange},
};

#[derive(Debug)]
pub struct CameraControllingEntity {
    eye_position: Point3<f32>,
}

impl CameraControllingEntity {
    pub fn new() -> Self {
        Self {
            eye_position: Camera::DEFAULT_CAMERA_EYE_POSITION.into(),
        }
    }
}

impl Default for CameraControllingEntity {
    fn default() -> Self {
        Self {
            eye_position: Camera::DEFAULT_CAMERA_EYE_POSITION.into(),
        }
    }
}

impl TEntity for CameraControllingEntity {
    fn get_entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new("Camera Controlling Entity", UpdateFrequency::Fast, false)
    }

    fn update(&mut self, delta_time: f64, input_handler: &InputHandler) -> Vec<EntityAction> {
        if input_handler.is_key_pressed(&VirtualKeyCode::W) {
            self.eye_position += Vector3::new(0.0, 0.0, (0.1 * delta_time) as f32);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::S) {
            self.eye_position += Vector3::new(0.0, 0.0, (-0.1 * delta_time) as f32);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::A) {
            self.eye_position += Vector3::new((0.1 * delta_time) as f32, 0.0, 0.0);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::D) {
            self.eye_position += Vector3::new((-0.1 * delta_time) as f32, 0.0, 0.0);
        }

        vec![EntityAction::CameraChange(
            CameraChange::new().with_eye(self.eye_position),
        )]
    }
}
