use std::collections::HashSet;

use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    window::Window,
};

use super::mouse_input_handler::MouseInputHandler;

#[derive(Debug, Default)]
pub struct KeyboardInputHandler {
    pressed: HashSet<VirtualKeyCode>,
}

impl KeyboardInputHandler {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    fn post_update_mouse_handler(&self, mouse_input_handler: &mut MouseInputHandler) {
        if self.is_pressed(&VirtualKeyCode::LAlt) {
            mouse_input_handler.set_hide_mouse_if_grabbed(false);
            mouse_input_handler.set_should_grab(false);
            mouse_input_handler.set_reset_cursor_to_center(false);
        } else {
            mouse_input_handler.set_hide_mouse_if_grabbed(true);
            mouse_input_handler.set_should_grab(true);
            mouse_input_handler.set_reset_cursor_to_center(true);
        }
    }

    fn post_update_exit_listener(&self) -> bool {
        self.is_pressed(&VirtualKeyCode::Escape)
    }

    /// Returns 'true' if the app should be exited/closed.
    pub fn post_update(
        &self,
        _window: &mut Window,
        mouse_input_handler: &mut MouseInputHandler,
    ) -> bool {
        self.post_update_mouse_handler(mouse_input_handler);
        self.post_update_exit_listener()
    }

    pub fn handle_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
                // Push pressed key to vec
                self.pressed.insert(keycode);
            } else {
                self.pressed.remove(&keycode);
            }
        }
    }

    pub fn is_pressed(&self, pressed_key: &VirtualKeyCode) -> bool {
        self.pressed.contains(pressed_key)
    }

    pub fn are_all_pressed(&self, pressed_keys: &[VirtualKeyCode]) -> bool {
        let mut r_vec: Vec<bool> = Vec::new();

        for pressed_key in pressed_keys {
            let r = self.pressed.contains(pressed_key);
            r_vec.push(r);
        }

        r_vec.iter().all(|&x| x)
    }

    pub fn is_any_pressed(&self, pressed_keys: &[VirtualKeyCode]) -> bool {
        let mut r_vec: Vec<bool> = Vec::new();

        for pressed_key in pressed_keys {
            let r = self.pressed.contains(pressed_key);
            r_vec.push(r);
        }

        r_vec.iter().any(|&x| x)
    }
}
