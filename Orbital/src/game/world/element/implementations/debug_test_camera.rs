use cgmath::Point3;
use gilrs::{Axis, Button};
use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    app::InputEvent,
    game::{Element, ElementRegistration, WorldChange},
    resources::descriptors::CameraDescriptor,
};

pub struct DebugTestCamera {
    axis_forward_back: f32,
    axis_left_right: f32,
    axis_up_down: f32,
    is_axis_input: bool,
    camera_change: CameraDescriptor,
}

impl DebugTestCamera {
    pub const DEBUG_CAMERA_NAME: &'static str = "DEBUG";

    pub const KEY_MOVE_FORWARD: PhysicalKey = PhysicalKey::Code(KeyCode::KeyW);
    pub const KEY_MOVE_BACKWARD: PhysicalKey = PhysicalKey::Code(KeyCode::KeyS);
    pub const KEY_MOVE_LEFT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyA);
    pub const KEY_MOVE_RIGHT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyD);
    pub const KEY_MOVE_DOWN: PhysicalKey = PhysicalKey::Code(KeyCode::KeyQ);
    pub const KEY_MOVE_UP: PhysicalKey = PhysicalKey::Code(KeyCode::KeyE);

    pub fn new() -> Self {
        Self {
            axis_forward_back: 0.0,
            axis_left_right: 0.0,
            axis_up_down: 0.0,
            is_axis_input: false,
            camera_change: CameraDescriptor {
                position: Point3::new(5.0, 0.0, 0.0),
                ..Default::default()
            },
        }
    }
}

impl Element for DebugTestCamera {
    fn on_registration(&mut self, _ulid: &ulid::Ulid) -> ElementRegistration {
        ElementRegistration {
            tags: Some(vec!["debug test camera".into()]),
            world_changes: Some(vec![WorldChange::SpawnCameraAndMakeActive(
                self.camera_change.clone(),
            )]),
            ..Default::default()
        }
    }

    fn on_input_event(&mut self, _delta_time: f64, input_event: &InputEvent) {
        match input_event {
            InputEvent::KeyboardButton {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event.physical_key {
                Self::KEY_MOVE_FORWARD => {
                    self.axis_forward_back = if event.state == ElementState::Pressed {
                        -1.0
                    } else {
                        0.0
                    };
                    self.is_axis_input = false;
                }
                Self::KEY_MOVE_BACKWARD => {
                    self.axis_forward_back = if event.state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    };
                    self.is_axis_input = false;
                }
                Self::KEY_MOVE_LEFT => {
                    self.axis_left_right = if event.state == ElementState::Pressed {
                        -1.0
                    } else {
                        0.0
                    };
                    self.is_axis_input = false;
                }
                Self::KEY_MOVE_RIGHT => {
                    self.axis_left_right = if event.state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    };
                    self.is_axis_input = false;
                }
                Self::KEY_MOVE_DOWN => {
                    self.axis_up_down = if event.state == ElementState::Pressed {
                        -1.0
                    } else {
                        0.0
                    };
                    self.is_axis_input = false;
                }
                Self::KEY_MOVE_UP => {
                    self.axis_up_down = if event.state == ElementState::Pressed {
                        1.0
                    } else {
                        0.0
                    };
                    self.is_axis_input = false;
                }
                _ => (),
            },
            InputEvent::GamepadAxis {
                gamepad_id: _,
                axis,
                value,
            } => match axis {
                Axis::LeftStickX => {
                    if *value > 0.15 {
                        self.axis_left_right = *value;
                        self.is_axis_input = true;
                    } else if *value < -0.15 {
                        self.axis_left_right = *value;
                        self.is_axis_input = true;
                    } else if self.is_axis_input {
                        self.axis_left_right = 0.0;
                        self.is_axis_input = true;
                    }
                }
                Axis::LeftStickY => {
                    if *value > 0.15 {
                        self.axis_forward_back = -*value;
                        self.is_axis_input = true;
                    } else if *value < -0.15 {
                        self.axis_forward_back = -*value;
                        self.is_axis_input = true;
                    } else if self.is_axis_input {
                        self.axis_forward_back = 0.0;
                        self.is_axis_input = true;
                    }
                }
                _ => (),
            },
            InputEvent::GamepadButton {
                gamepad_id: _,
                button,
                pressed,
            } => match button {
                Button::DPadUp => {
                    self.axis_up_down = if *pressed { 1.0 } else { 0.0 };
                    self.is_axis_input = false;
                }
                Button::DPadDown => {
                    self.axis_up_down = if *pressed { -1.0 } else { 0.0 };
                    self.is_axis_input = false;
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        self.camera_change.position.x += self.axis_forward_back * delta_time as f32;
        self.camera_change.position.z += self.axis_left_right * delta_time as f32;
        self.camera_change.position.y += self.axis_up_down * delta_time as f32;

        Some(vec![WorldChange::UpdateCamera(self.camera_change.clone())])
    }
}
