use winit::event::VirtualKeyCode;

use crate::{
    app::{EntityAction, EntityConfiguration, InputHandler, TEntity, UpdateFrequency},
    engine::CameraChange,
};

#[derive(Debug)]
pub struct CameraControllingEntity {}

impl CameraControllingEntity {
    pub fn new() -> Self {
        Self {}
    }
}

impl CameraControllingEntity {
    fn handle_keyboard_input(
        &self,
        input_handler: &InputHandler,
        camera_change: &mut CameraChange,
    ) {
        if input_handler
            .keyboard_input_handler()
            .is_pressed(&VirtualKeyCode::W)
        {
            camera_change.with_amount_forward(1.0);
        }

        if input_handler
            .keyboard_input_handler()
            .is_pressed(&VirtualKeyCode::S)
        {
            camera_change.with_amount_backward(1.0);
        }

        if input_handler
            .keyboard_input_handler()
            .is_pressed(&VirtualKeyCode::A)
        {
            camera_change.with_amount_left(1.0);
        }

        if input_handler
            .keyboard_input_handler()
            .is_pressed(&VirtualKeyCode::D)
        {
            camera_change.with_amount_right(1.0);
        }

        if input_handler
            .keyboard_input_handler()
            .is_pressed(&VirtualKeyCode::Space)
        {
            camera_change.with_amount_up(1.0);
        }

        if input_handler
            .keyboard_input_handler()
            .is_pressed(&VirtualKeyCode::LShift)
        {
            camera_change.with_amount_down(1.0);
        }
    }

    fn handle_mouse(&mut self, input_handler: &InputHandler, camera_change: &mut CameraChange) {
        let (x, y) = &input_handler.mouse_input_handler().cursor_position();

        camera_change.with_rotate_horizontal(*x as f32);
        camera_change.with_rotate_vertical(*y as f32);
    }
}

impl TEntity for CameraControllingEntity {
    fn entity_configuration(&self) -> EntityConfiguration {
        EntityConfiguration::new("Camera Controlling Entity", UpdateFrequency::Fast, false)
    }

    fn update(&mut self, _delta_time: f64, input_handler: &InputHandler) -> Vec<EntityAction> {
        let mut camera_change = CameraChange::new();

        self.handle_keyboard_input(input_handler, &mut camera_change);
        self.handle_mouse(input_handler, &mut camera_change);

        vec![EntityAction::CameraChange(camera_change)]
    }
}

impl Default for CameraControllingEntity {
    fn default() -> Self {
        Self::new()
    }
}
