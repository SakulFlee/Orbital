#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputAxis {
    MouseMovement,
    MouseScrollWheel,
    #[cfg(feature = "gamepad_input")]
    GamepadLeftStick,
    #[cfg(feature = "gamepad_input")]
    GamepadRightStick,
    #[cfg(feature = "gamepad_input")]
    GamepadTrigger,
}
