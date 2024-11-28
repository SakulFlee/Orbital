#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputAxis {
    /// X & Y will be mapped to the actual mouse movement delta vector.  
    /// Both axis can be positive and negative.  
    /// Both axis might be beyond [-]1.0.
    MouseMovement,
    /// X & Y will be mapped to the actual mouse scroll wheel delta vector.  
    /// Both axis can be positive and negative.  
    /// Both axis might be beyond [-]1.0.
    MouseScrollWheel,
    /// X & Y will be mapped to the gamepads left stick.  
    /// Both axis can be positive and negative.  
    /// Both axis should be within -1.0 to +1.0 range.
    #[cfg(feature = "gamepad_input")]
    GamepadLeftStick,
    /// X & Y will be mapped to the gamepads left stick.  
    /// Both axis can be positive and negative.  
    /// Both axis should be within -1.0 to +1.0 range.
    #[cfg(feature = "gamepad_input")]
    GamepadRightStick,
    /// X will be mapped to the left Z trigger.
    /// Y will be mapped to the right Z trigger.
    /// Both axis can only ever be positive.
    /// Both axis should be within 0.0 to +1.0 range.
    #[cfg(feature = "gamepad_input")]
    GamepadTrigger,
}
