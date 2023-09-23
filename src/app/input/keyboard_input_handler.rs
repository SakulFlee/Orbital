use std::collections::HashSet;

use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

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
