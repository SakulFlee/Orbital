use winit::event::DeviceId;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputId {
    KeyboardOrMouse(DeviceId),
    /// Touch input (finger/pen on a touchscreen).
    Touch(DeviceId),
    /// A gamepad, identified by a backend-assigned number: gilrs'
    /// `GamepadId` index on desktop, a platform device id (0 for the
    /// single multiplexed pad) on mobile.
    #[cfg(feature = "gamepad_input")]
    Gamepad(u32),
}
