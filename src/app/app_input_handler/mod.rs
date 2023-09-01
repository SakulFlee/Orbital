use winit::event::VirtualKeyCode;

use self::keyboard_input_handler::AppKeyboardInputHandler;

pub mod controller_input_handler;
pub mod keyboard_input_handler;
pub mod mouse_input_handler;

#[derive(Debug, Default, Hash)]
pub struct AppInputHandler {
    keyboard_input_handler: AppKeyboardInputHandler,
}

impl AppInputHandler {
    pub fn new() -> Self {
        Self {
            keyboard_input_handler: AppKeyboardInputHandler::new(),
        }
    }

    pub fn get_keyboard_input_handler(&mut self) -> &mut AppKeyboardInputHandler {
        &mut self.keyboard_input_handler
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
