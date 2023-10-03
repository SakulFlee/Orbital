use winit::event::VirtualKeyCode;

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

    pub fn keyboard_input_handler(&mut self) -> &mut KeyboardInputHandler {
        &mut self.keyboard_input_handler
    }

    pub fn mouse_input_handler(&mut self) -> &mut MouseInputHandler {
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
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
