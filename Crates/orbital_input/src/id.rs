use winit::event::DeviceId;

#[cfg(feature = "gamepad_input")]
use gilrs::GamepadId;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputId {
    KeyboardOrMouse(DeviceId),
    #[cfg(feature = "gamepad_input")]
    Gamepad(GamepadId),
}
