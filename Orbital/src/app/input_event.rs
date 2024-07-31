use std::fmt::Debug;

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
}
