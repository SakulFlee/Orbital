use winit::event::{MouseScrollDelta, TouchPhase, VirtualKeyCode};

use self::{keyboard_input_handler::KeyboardInputHandler, mouse_input_handler::MouseInputHandler};

pub mod controller_input_handler;
pub mod keyboard_input_handler;
pub mod mouse_input_handler;

#[derive(Debug)]
pub struct InputHandler {
    keyboard_input_handler: KeyboardInputHandler,
    mouse_input_handler: MouseInputHandler,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            keyboard_input_handler: KeyboardInputHandler::new(),
            mouse_input_handler: MouseInputHandler::new(),
        }
    }

    pub fn keyboard_input_handler(&self) -> &KeyboardInputHandler {
        &self.keyboard_input_handler
    }

    pub fn keyboard_input_handler_mut(&mut self) -> &mut KeyboardInputHandler {
        &mut self.keyboard_input_handler
    }

    pub fn mouse_input_handler(&mut self) -> &MouseInputHandler {
        &self.mouse_input_handler
    }

    pub fn mouse_input_handler_mut(&mut self) -> &mut MouseInputHandler {
        &mut self.mouse_input_handler
    }

    pub fn is_key_pressed(&self, pressed_key: &VirtualKeyCode) -> bool {
        self.keyboard_input_handler.is_pressed(pressed_key)
    }

    pub fn are_all_keys_pressed(&self, pressed_keys: &[VirtualKeyCode]) -> bool {
        self.keyboard_input_handler.are_all_pressed(pressed_keys)
    }

    pub fn is_any_key_pressed(&self, pressed_keys: &[VirtualKeyCode]) -> bool {
        self.keyboard_input_handler.is_any_pressed(pressed_keys)
    }

    pub fn cursor_position(&self) -> (f64, f64) {
        self.mouse_input_handler.cursor_position()
    }

    pub fn is_inside(&self) -> bool {
        self.mouse_input_handler.is_inside()
    }

    pub fn lmb_pressed(&self) -> bool {
        self.mouse_input_handler.lmb_pressed()
    }

    pub fn rmb_pressed(&self) -> bool {
        self.mouse_input_handler.rmb_pressed()
    }

    pub fn mmb_pressed(&self) -> bool {
        self.mouse_input_handler.mmb_pressed()
    }

    pub fn scroll(&self) -> Option<(TouchPhase, MouseScrollDelta)> {
        self.mouse_input_handler.scroll()
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
