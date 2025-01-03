use std::fmt::Debug;

use gilrs::{Axis, Button, Event, EventType, GamepadId};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
};

/// A mix of [winit::event::WindowEvent] and [winit::event::DeviceEvent] to be used by [crate::app::App]s during [crate::app::App::on_input].
///
/// For more details, check [winit::event::WindowEvent] and [winit::event::DeviceEvent]
#[derive(Debug)]
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
    GamepadButton {
        gamepad_id: GamepadId,
        button: Button,
        pressed: bool,
    },
    GamepadAxis {
        gamepad_id: GamepadId,
        axis: Axis,
        value: f32,
    },
    GamepadConnected {
        gamepad_id: GamepadId,
    },
    GamepadDisconnected {
        gamepad_id: GamepadId,
    },
}

impl InputEvent {
    pub fn convert(gil_event: Event) -> Option<Self> {
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
