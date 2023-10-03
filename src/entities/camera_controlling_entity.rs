use winit::event::VirtualKeyCode;

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::CameraChange,
};

#[derive(Debug)]
pub struct CameraControllingEntity {
    speed: f32,
    sensitivity: f32,
}

impl CameraControllingEntity {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self { speed, sensitivity }
    }
}

impl CameraControllingEntity {
    fn handle_keyboard_input(
        &self,
        input_handler: &InputHandler,
        camera_change: &mut CameraChange,
    ) {
        if input_handler.is_key_pressed(&VirtualKeyCode::W) {
            camera_change.with_amount_forward(1.0);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::S) {
            camera_change.with_amount_backward(1.0);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::A) {
            camera_change.with_amount_left(1.0);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::D) {
            camera_change.with_amount_right(1.0);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::Space) {
            camera_change.with_amount_up(1.0);
        }

        if input_handler.is_key_pressed(&VirtualKeyCode::LShift) {
            camera_change.with_amount_down(1.0);
        }
    }

    fn handle_mouse(&self, input_handler: &InputHandler, camera_change: &mut CameraChange) {
        todo!()
    }
}

impl TEntity for CameraControllingEntity {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new("Camera Controlling Entity", UpdateFrequency::Fast, false)
    }

    fn update(&mut self, delta_time: f64, input_handler: &InputHandler) -> Vec<EntityAction> {
        let mut camera_change = CameraChange::new();

        self.handle_keyboard_input(input_handler, &mut camera_change);
        self.handle_mouse(input_handler, &mut camera_change);

        vec![EntityAction::CameraChange(camera_change)]
    }
}
