use winit::{event::MouseButton, keyboard::PhysicalKey};

#[cfg(feature = "gamepad_input")]
use gilrs::Button;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputButton {
    Keyboard(PhysicalKey),
    Mouse(MouseButton),
    #[cfg(feature = "gamepad_input")]
    Gamepad(Button),
}
