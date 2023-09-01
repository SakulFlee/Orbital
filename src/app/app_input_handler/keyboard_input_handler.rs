use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Debug, Default, Hash)]
pub struct AppKeyboardInputHandler {
    pressed: Vec<VirtualKeyCode>,
}

impl AppKeyboardInputHandler {
    pub fn new() -> Self {
        Self {
            pressed: Vec::new(),
        }
    }

    pub fn handle_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
                // Push pressed key to vec
                self.pressed.push(keycode);
            } else {
                // index counter
                let mut i = 0;
                let mut found = false;
                // For each pressed key ...
                for pressed_key in self.pressed.iter() {
                    // ... check if the keycode matches ...
                    if *pressed_key == keycode {
                        // ... and break the loop if found!
                        found = true;
                        break;
                    }
                    // increment index counter
                    i += 1;
                }

                // Remove the key from the vec
                if found {
                    self.pressed.remove(i);
                }
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
