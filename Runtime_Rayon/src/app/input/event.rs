use std::fmt::Debug;

use gilrs::{Axis, Button, EventType, GamepadId};
use winit::{
    dpi::PhysicalPosition,
    event::{
        DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase,
    },
};

/// A mix of [winit::event::WindowEvent], [winit::event::DeviceEvent] and [gilrs::Event] (if enabled) to be used by [crate::app::App]s during [crate::app::App::on_input].
///
/// For more details, check [winit::event::WindowEvent] and [winit::event::DeviceEvent]
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
    DeviceConnected {
        device_id: DeviceId,
    },
    DeviceDisconnected {
        device_id: DeviceId,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadButton {
        gamepad_id: GamepadId,
        button: Button,
        pressed: bool,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadAxis {
        gamepad_id: GamepadId,
        axis: Axis,
        value: f32,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadConnected {
        gamepad_id: GamepadId,
    },
    #[cfg(feature = "gamepad_input")]
    GamepadDisconnected {
        gamepad_id: GamepadId,
    },
}

#[cfg(feature = "gamepad_input")]
impl InputEvent {
    pub fn convert_device_event(device_id: DeviceId, device_event: DeviceEvent) -> Option<Self> {
        match device_event {
            DeviceEvent::Added => Some(Self::DeviceConnected { device_id }),
            DeviceEvent::Removed => Some(Self::DeviceDisconnected { device_id }),
            DeviceEvent::MouseMotion { delta } => {
                Some(InputEvent::MouseMovedDelta { device_id, delta })
            }
            _ => None,
        }
    }

    pub fn convert_gil_event(gil_event: gilrs::Event) -> Option<Self> {
        match gil_event.event {
            EventType::ButtonPressed(button, _) => Some(Self::GamepadButton {
                gamepad_id: gil_event.id,
                button,
                pressed: true,
            }),
            EventType::ButtonRepeated(button, _) => Some(Self::GamepadButton {
                gamepad_id: gil_event.id,
                button,
                pressed: true,
            }),
            EventType::ButtonReleased(button, _) => Some(Self::GamepadButton {
                gamepad_id: gil_event.id,
                button,
                pressed: false,
            }),
            EventType::AxisChanged(axis, value, _) => Some(Self::GamepadAxis {
                gamepad_id: gil_event.id,
                axis,
                value,
            }),
            EventType::Connected => Some(Self::GamepadConnected {
                gamepad_id: gil_event.id,
            }),
            EventType::Disconnected => Some(Self::GamepadDisconnected {
                gamepad_id: gil_event.id,
            }),
            EventType::Dropped => Some(Self::GamepadDisconnected {
                gamepad_id: gil_event.id,
            }),
            _ => None,
        }
    }
}
