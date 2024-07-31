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
    MouseMoved {
        device_id: DeviceId,
        position: PhysicalPosition<f64>,
    },
    GamepadButton {
        gamepad_id: GamepadId,
        button: Button,
        pressed: bool,
    },
    GamepadButtonValue {
        gamepad_id: GamepadId,
        button: Button,
        value: f32,
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

impl From<Event> for InputEvent {
    fn from(event: Event) -> Self {
        match event.event {
            EventType::ButtonPressed(button, _) => Self::GamepadButton {
                gamepad_id: event.id,
                button: button,
                pressed: true,
            },
            EventType::ButtonRepeated(button, _) => Self::GamepadButton {
                gamepad_id: event.id,
                button: button,
                pressed: true,
            },
            EventType::ButtonReleased(button, _) => Self::GamepadButton {
                gamepad_id: event.id,
                button: button,
                pressed: false,
            },
            EventType::AxisChanged(axis, value, _) => Self::GamepadAxis {
                gamepad_id: event.id,
                axis: axis,
                value: value,
            },
            EventType::Connected => Self::GamepadConnected {
                gamepad_id: event.id,
            },
            EventType::Disconnected => Self::GamepadDisconnected {
                gamepad_id: event.id,
            },
            EventType::Dropped => Self::GamepadDisconnected {
                gamepad_id: event.id,
            },
            EventType::ButtonChanged(button, value, _) => Self::GamepadButtonValue {
                gamepad_id: event.id,
                button: button,
                value: value,
            },
        }
    }
}
