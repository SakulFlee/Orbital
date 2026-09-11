use winit::event::DeviceId;

#[cfg(feature = "gamepad_input")]
use gilrs::GamepadId;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputId {
    KeyboardOrMouse(DeviceId),
    /// Touch input (finger/pen on a touchscreen).
    Touch(DeviceId),
    #[cfg(feature = "gamepad_input")]
    Gamepad(GamepadId),
}
