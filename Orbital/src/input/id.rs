use winit::event::DeviceId;

#[cfg(feature = "gamepad_input")]
use gilrs::GamepadId;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputId {
    /// Specifies a Keyboard or Mouse device.
    ///
    /// Note:
    /// Mouse and Keyboards aren't separated!
    /// However, a Mouse can never trigger a Keyboard event.
    KeyboardOrMouse(DeviceId),
    #[cfg(feature = "gamepad_input")]
    Gamepad(GamepadId),
}
