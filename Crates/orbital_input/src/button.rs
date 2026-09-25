use winit::{event::MouseButton, keyboard::PhysicalKey};

/// A gamepad button in semantic (SDL-style) form.
///
/// Mirrors the gilrs/SDL button set so every backend speaks one
/// vocabulary: gilrs reports these on desktop, and mobile platform
/// providers (Android `KEYCODE_BUTTON_*`, iOS `GCController`) translate
/// into them at the event boundary.
///
/// The `#[repr(u32)]` discriminants are the engine's device-event ABI:
/// mobile backends report buttons as these codes through
/// `winit::event::DeviceEvent::Button`.
#[cfg(feature = "gamepad_input")]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
#[repr(u32)]
pub enum GamepadButton {
    // Action Pad
    South = 0,
    East = 1,
    North = 2,
    West = 3,
    C = 4,
    Z = 5,
    // Triggers
    LeftTrigger = 6,
    LeftTrigger2 = 7,
    RightTrigger = 8,
    RightTrigger2 = 9,
    // Menu Pad
    Select = 10,
    Start = 11,
    Mode = 12,
    // Sticks
    LeftThumb = 13,
    RightThumb = 14,
    // D-Pad
    DPadUp = 15,
    DPadDown = 16,
    DPadLeft = 17,
    DPadRight = 18,
    #[default]
    Unknown = 19,
}

#[cfg(feature = "gamepad_input")]
impl GamepadButton {
    /// Decode a device-event ABI button code. `None` for unknown codes.
    pub fn from_abi(code: u32) -> Option<Self> {
        Some(match code {
            0 => Self::South,
            1 => Self::East,
            2 => Self::North,
            3 => Self::West,
            4 => Self::C,
            5 => Self::Z,
            6 => Self::LeftTrigger,
            7 => Self::LeftTrigger2,
            8 => Self::RightTrigger,
            9 => Self::RightTrigger2,
            10 => Self::Select,
            11 => Self::Start,
            12 => Self::Mode,
            13 => Self::LeftThumb,
            14 => Self::RightThumb,
            15 => Self::DPadUp,
            16 => Self::DPadDown,
            17 => Self::DPadLeft,
            18 => Self::DPadRight,
            19 => Self::Unknown,
            _ => return None,
        })
    }

    pub fn is_action(self) -> bool {
        use GamepadButton::*;
        matches!(self, South | East | North | West | C | Z)
    }

    pub fn is_trigger(self) -> bool {
        use GamepadButton::*;
        matches!(
            self,
            LeftTrigger | LeftTrigger2 | RightTrigger | RightTrigger2
        )
    }

    pub fn is_menu(self) -> bool {
        use GamepadButton::*;
        matches!(self, Select | Start | Mode)
    }

    pub fn is_stick(self) -> bool {
        use GamepadButton::*;
        matches!(self, LeftThumb | RightThumb)
    }

    pub fn is_dpad(self) -> bool {
        use GamepadButton::*;
        matches!(self, DPadUp | DPadDown | DPadLeft | DPadRight)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputButton {
    Keyboard(PhysicalKey),
    Mouse(MouseButton),
    /// A finger touching the screen, identified by its winit touch id.
    Touch(u64),
    #[cfg(feature = "gamepad_input")]
    Gamepad(GamepadButton),
}
