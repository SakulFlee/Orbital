use std::fmt::Debug;

#[cfg(feature = "gamepad_input")]
use cgmath::Vector2;
#[cfg(feature = "gamepad_gilrs")]
use gilrs::{Axis, Button, EventType};
use winit::{
    dpi::PhysicalPosition,
    event::{
        DeviceEvent, DeviceId, ElementState, Force, KeyEvent, MouseButton, MouseScrollDelta,
        TouchPhase,
    },
};

#[cfg(feature = "gamepad_input")]
use crate::GamepadButton;
use crate::InputAxis;

/// Device-event axis codes shared with our winit fork's mobile joystick
/// translation — the engine's device-event ABI for
/// `winit::event::DeviceEvent::Motion`.
#[cfg(all(
    any(target_os = "android", target_os = "ios"),
    feature = "gamepad_input"
))]
const MOTION_AXIS_LEFT_STICK_X: u32 = 0;
#[cfg(all(
    any(target_os = "android", target_os = "ios"),
    feature = "gamepad_input"
))]
const MOTION_AXIS_LEFT_STICK_Y: u32 = 1;
#[cfg(all(
    any(target_os = "android", target_os = "ios"),
    feature = "gamepad_input"
))]
const MOTION_AXIS_RIGHT_STICK_X: u32 = 2;
#[cfg(all(
    any(target_os = "android", target_os = "ios"),
    feature = "gamepad_input"
))]
const MOTION_AXIS_RIGHT_STICK_Y: u32 = 3;
#[cfg(all(
    any(target_os = "android", target_os = "ios"),
    feature = "gamepad_input"
))]
const MOTION_AXIS_TRIGGER_LEFT: u32 = 4;
#[cfg(all(
    any(target_os = "android", target_os = "ios"),
    feature = "gamepad_input"
))]
const MOTION_AXIS_TRIGGER_RIGHT: u32 = 5;

/// A mix of [winit::event::WindowEvent], [winit::event::DeviceEvent] and
/// [gilrs::Event] (with the `gamepad_gilrs` feature) to be used by App
/// implementations. Gamepad variants carry the engine's semantic types —
/// backends translate into this vocabulary at the event boundary.
#[derive(Debug, PartialEq)]
pub enum InputEvent {
    KeyboardButton {
        device_id: DeviceId,
        event: KeyEvent,
        is_synthetic: bool,
    },
    MouseButton {
        device_id: DeviceId,
        state: ElementState,
        button: MouseButton,
    },
    MouseWheel {
        device_id: DeviceId,
        delta: MouseScrollDelta,
        phase: TouchPhase,
    },
    MouseMovedPosition {
        device_id: DeviceId,
        position: PhysicalPosition<f64>,
    },
    MouseMovedDelta {
        device_id: DeviceId,
        delta: (f64, f64),
    },
    Touch {
        device_id: DeviceId,
        phase: TouchPhase,
        location: PhysicalPosition<f64>,
        id: u64,
        force: Option<Force>,
    },
    DeviceConnected {
        device_id: DeviceId,
    },
    DeviceDisconnected {
        device_id: DeviceId,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadButton {
        gamepad_id: u32,
        button: GamepadButton,
        pressed: bool,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadAxis {
        gamepad_id: u32,
        axis: InputAxis,
        /// Absolute stick/trigger position, shaped as `(x, 0)` / `(0, y)`
        /// per reported component; `InputState` stores it as the current
        /// deflection (unlike mouse deltas, which accumulate per frame).
        delta: Vector2<f64>,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadConnected {
        gamepad_id: u32,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadDisconnected {
        gamepad_id: u32,
    },
}

impl InputEvent {
    pub fn convert_device_event(device_id: DeviceId, device_event: DeviceEvent) -> Option<Self> {
        match device_event {
            DeviceEvent::Added => Some(Self::DeviceConnected { device_id }),
            DeviceEvent::Removed => Some(Self::DeviceDisconnected { device_id }),
            DeviceEvent::MouseMotion { delta } => {
                Some(InputEvent::MouseMovedDelta { device_id, delta })
            }
            // Mobile joystick translation (our winit fork): gamepad axes
            // and buttons arrive as generic device events carrying
            // engine-ABI u32 codes. Gated to mobile because desktop
            // backends report mouse-relative data through Motion/Button,
            // which must not be read as gamepad input.
            #[cfg(all(
                any(target_os = "android", target_os = "ios"),
                feature = "gamepad_input"
            ))]
            DeviceEvent::Motion { axis, value } => {
                let (input_axis, delta) = match axis {
                    MOTION_AXIS_LEFT_STICK_X => {
                        (InputAxis::GamepadLeftStick, Vector2::new(value, 0.0))
                    }
                    MOTION_AXIS_LEFT_STICK_Y => {
                        (InputAxis::GamepadLeftStick, Vector2::new(0.0, value))
                    }
                    MOTION_AXIS_RIGHT_STICK_X => {
                        (InputAxis::GamepadRightStick, Vector2::new(value, 0.0))
                    }
                    MOTION_AXIS_RIGHT_STICK_Y => {
                        (InputAxis::GamepadRightStick, Vector2::new(0.0, value))
                    }
                    MOTION_AXIS_TRIGGER_LEFT => {
                        (InputAxis::GamepadTrigger, Vector2::new(value, 0.0))
                    }
                    MOTION_AXIS_TRIGGER_RIGHT => {
                        (InputAxis::GamepadTrigger, Vector2::new(0.0, value))
                    }
                    _ => return None,
                };
                // Mobile multiplexes its pads behind a single device id
                // (winit's DeviceId has no stable numeric form), so the
                // first gamepad is id 0.
                Some(Self::GamepadAxis {
                    gamepad_id: 0,
                    axis: input_axis,
                    delta,
                })
            }
            #[cfg(all(
                any(target_os = "android", target_os = "ios"),
                feature = "gamepad_input"
            ))]
            DeviceEvent::Button { button, state } => Some(Self::GamepadButton {
                gamepad_id: 0,
                button: GamepadButton::from_abi(button)?,
                pressed: state == ElementState::Pressed,
            }),
            _ => None,
        }
    }

    #[cfg(feature = "gamepad_gilrs")]
    pub fn convert_gil_event(gil_event: gilrs::Event) -> Option<Self> {
        let gamepad_id = u32::try_from(usize::from(gil_event.id)).unwrap_or(u32::MAX);
        match gil_event.event {
            EventType::ButtonPressed(button, _) | EventType::ButtonRepeated(button, _) => {
                Some(Self::GamepadButton {
                    gamepad_id,
                    button: convert_gil_button(button),
                    pressed: true,
                })
            }
            EventType::ButtonReleased(button, _) => Some(Self::GamepadButton {
                gamepad_id,
                button: convert_gil_button(button),
                pressed: false,
            }),
            EventType::AxisChanged(axis, value, _) => {
                let (input_axis, delta) = match axis {
                    Axis::LeftStickX => {
                        (InputAxis::GamepadLeftStick, Vector2::new(value as f64, 0.0))
                    }
                    Axis::LeftStickY => {
                        (InputAxis::GamepadLeftStick, Vector2::new(0.0, value as f64))
                    }
                    Axis::RightStickX => (
                        InputAxis::GamepadRightStick,
                        Vector2::new(value as f64, 0.0),
                    ),
                    Axis::RightStickY => (
                        InputAxis::GamepadRightStick,
                        Vector2::new(0.0, value as f64),
                    ),
                    Axis::LeftZ => (InputAxis::GamepadTrigger, Vector2::new(value as f64, 0.0)),
                    Axis::RightZ => (InputAxis::GamepadTrigger, Vector2::new(0.0, value as f64)),
                    // D-pad axes and unknown axes are not modeled — D-pad
                    // presses arrive as button events instead.
                    _ => return None,
                };
                Some(Self::GamepadAxis {
                    gamepad_id,
                    axis: input_axis,
                    delta,
                })
            }
            EventType::Connected => Some(Self::GamepadConnected { gamepad_id }),
            EventType::Disconnected | EventType::Dropped => {
                Some(Self::GamepadDisconnected { gamepad_id })
            }
            _ => None,
        }
    }
}

/// Mechanical 1:1 translation of gilrs' button set into the semantic set.
#[cfg(feature = "gamepad_gilrs")]
fn convert_gil_button(button: Button) -> GamepadButton {
    match button {
        Button::South => GamepadButton::South,
        Button::East => GamepadButton::East,
        Button::North => GamepadButton::North,
        Button::West => GamepadButton::West,
        Button::C => GamepadButton::C,
        Button::Z => GamepadButton::Z,
        Button::LeftTrigger => GamepadButton::LeftTrigger,
        Button::LeftTrigger2 => GamepadButton::LeftTrigger2,
        Button::RightTrigger => GamepadButton::RightTrigger,
        Button::RightTrigger2 => GamepadButton::RightTrigger2,
        Button::Select => GamepadButton::Select,
        Button::Start => GamepadButton::Start,
        Button::Mode => GamepadButton::Mode,
        Button::LeftThumb => GamepadButton::LeftThumb,
        Button::RightThumb => GamepadButton::RightThumb,
        Button::DPadUp => GamepadButton::DPadUp,
        Button::DPadDown => GamepadButton::DPadDown,
        Button::DPadLeft => GamepadButton::DPadLeft,
        Button::DPadRight => GamepadButton::DPadRight,
        Button::Unknown => GamepadButton::Unknown,
    }
}
