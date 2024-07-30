use std::fmt::Debug;

use winit::{
    dpi::PhysicalPosition,
    event::{
        AxisId, ButtonId, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta,
        TouchPhase,
    },
    platform::windows::DeviceIdExtWindows,
};

/// A mix of [winit::event::WindowEvent] and [winit::event::DeviceEvent] to be used by [crate::app::App]s during [crate::app::App::on_input].
///
/// For more details, check [winit::event::WindowEvent] and [winit::event::DeviceEvent]
pub enum InputEvent {
    KeyboardInput {
        device_id: DeviceId,
        event: KeyEvent,
        is_synthetic: bool,
    },
    MouseInput {
        device_id: DeviceId,
        state: ElementState,
        button: MouseButton,
    },
    CursorEntered {
        device_id: DeviceId,
    },
    CursorLeft {
        device_id: DeviceId,
    },
    CursorMoved {
        device_id: DeviceId,
        position: PhysicalPosition<f64>,
    },
    AxisMotion {
        device_id: DeviceId,
        axis: AxisId,
        value: f64,
    },
    Added {
        device_id: DeviceId,
    },
    Removed {
        device_id: DeviceId,
    },
    MouseMotion {
        device_id: DeviceId,
        delta: (f64, f64),
    },
    MouseWheel {
        device_id: DeviceId,
        delta: MouseScrollDelta,
        phase: TouchPhase,
    },
    Button {
        device_id: DeviceId,
        button: ButtonId,
        state: ElementState,
    },
}

impl Debug for InputEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => f
                .debug_struct("KeyboardInput")
                .field("device_id", device_id)
                .field("event", event)
                .field("is_synthetic", is_synthetic)
                .finish(),
            Self::MouseInput {
                device_id,
                state,
                button,
            } => f
                .debug_struct("MouseInput")
                .field("device_id", device_id)
                .field("state", state)
                .field("button", button)
                .finish(),
            Self::CursorEntered { device_id } => f
                .debug_struct("CursorEntered")
                .field("device_id", device_id)
                .finish(),
            Self::CursorLeft { device_id } => f
                .debug_struct("CursorLeft")
                .field("device_id", device_id)
                .finish(),
            Self::CursorMoved {
                device_id,
                position,
            } => f
                .debug_struct("CursorMoved")
                .field("device_id", device_id)
                .field("position", position)
                .finish(),
            Self::AxisMotion {
                device_id,
                axis,
                value,
            } => f
                .debug_struct("AxisMotion")
                .field("device_id", device_id)
                .field("axis", axis)
                .field("value", value)
                .finish(),
            Self::Added { device_id } => f
                .debug_struct("Added")
                .field("device_id", device_id)
                .field("persistent_identifier", &device_id.persistent_identifier())
                .finish(),
            Self::Removed { device_id } => f
                .debug_struct("Removed")
                .field("device_id", device_id)
                .finish(),
            Self::MouseMotion { device_id, delta } => f
                .debug_struct("MouseMotion")
                .field("device_id", device_id)
                .field("delta", delta)
                .finish(),
            Self::MouseWheel {
                device_id,
                delta,
                phase,
            } => f
                .debug_struct("MouseWheel")
                .field("device_id", device_id)
                .field("delta", delta)
                .field("phase", phase)
                .finish(),
            Self::Button {
                device_id,
                button,
                state,
            } => f
                .debug_struct("Button")
                .field("device_id", device_id)
                .field("button", button)
                .field("state", state)
                .finish(),
        }
    }
}
