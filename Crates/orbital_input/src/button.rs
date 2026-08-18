use winit::{event::MouseButton, keyboard::PhysicalKey};

#[cfg(feature = "gamepad_input")]
use gilrs::Button;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputButton {
    Keyboard(PhysicalKey),
    Mouse(MouseButton),
    /// A finger touching the screen, identified by its winit touch id.
    Touch(u64),
    #[cfg(feature = "gamepad_input")]
    Gamepad(Button),
}
