use gilrs::{Axis, Button};
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta},
    keyboard::PhysicalKey,
};

use crate::app::InputEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDescriptor {
    KeyboardButton {
        key: PhysicalKey,
        pressed: bool,
    },
    MouseButton {
        button: MouseButton,
        pressed: bool,
    },
    MouseWheel {
        x_axis_increasing: Option<bool>,
        y_axis_increasing: Option<bool>,
    },
    GamepadButton {
        button: Button,
        pressed: bool,
    },
    GamepadAxis {
        axis: Axis,
        increasing: bool,
    },
}

impl InputDescriptor {
    pub fn convert(event: InputEvent, axis_threshold: f32) -> Option<Self> {
        match event {
            InputEvent::KeyboardButton {
                device_id: _,
                event,
                is_synthetic: _,
            } => Some(InputDescriptor::KeyboardButton {
                key: event.physical_key,
                pressed: event.state == ElementState::Pressed,
            }),
            InputEvent::MouseButton {
                device_id: _,
                state,
                button,
            } => Some(InputDescriptor::MouseButton {
                button,
                pressed: state == ElementState::Pressed,
            }),
            InputEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let (x, y) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(x) => (x.x as f32, x.y as f32),
                };

                let x_axis_increasing = if x >= axis_threshold {
                    Some(true)
                } else if x <= -axis_threshold {
                    Some(false)
                } else {
                    None
                };
                let y_axis_increasing = if y >= axis_threshold {
                    Some(true)
                } else if y <= -axis_threshold {
                    Some(false)
                } else {
                    None
                };

                Some(InputDescriptor::MouseWheel {
                    x_axis_increasing,
                    y_axis_increasing,
                })
            }
            InputEvent::GamepadButton {
                gamepad_id: _,
                button,
                pressed,
            } => Some(InputDescriptor::GamepadButton { button, pressed }),
            InputEvent::GamepadAxis {
                gamepad_id: _,
                axis,
                value,
            } => Some(InputDescriptor::GamepadAxis {
                axis,
                increasing: if value > axis_threshold {
                    true
                } else if value < -axis_threshold {
                    false
                } else {
                    return None;
                },
            }),
            _ => None,
        }
    }
}
