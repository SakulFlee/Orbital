use winit::window::Window;

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

    // Returns 'true' if the app should be exited.
    pub fn post_update(&mut self, window: &mut Window) -> bool {
        let exit = self
            .keyboard_input_handler
            .post_update(window, &mut self.mouse_input_handler);
        self.mouse_input_handler.post_update(window);

        exit
    }

    pub fn keyboard_input_handler(&self) -> &KeyboardInputHandler {
        &self.keyboard_input_handler
    }

    pub fn keyboard_input_handler_mut(&mut self) -> &mut KeyboardInputHandler {
        &mut self.keyboard_input_handler
    }

    pub fn mouse_input_handler(&self) -> &MouseInputHandler {
        &self.mouse_input_handler
    }

    pub fn mouse_input_handler_mut(&mut self) -> &mut MouseInputHandler {
        &mut self.mouse_input_handler
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
