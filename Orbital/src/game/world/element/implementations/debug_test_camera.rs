use cgmath::Point3;
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
    key_move_forward: bool,
    key_move_backward: bool,
    key_move_left: bool,
    key_move_right: bool,
    key_move_up: bool,
    key_move_down: bool,
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
            key_move_forward: false,
            key_move_backward: false,
            key_move_left: false,
            key_move_right: false,
            key_move_up: false,
            key_move_down: false,
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

    fn on_input_event(
        &mut self,
        _delta_time: f64,
        input_event: &InputEvent,
    ) -> Option<Vec<WorldChange>> {
        match input_event {
            InputEvent::KeyboardButton {
                device_id: _,
                event,
                is_synthetic: _,
            } => match event.physical_key {
                Self::KEY_MOVE_FORWARD => {
                    self.key_move_forward = event.state == ElementState::Pressed;
                }
                Self::KEY_MOVE_BACKWARD => {
                    self.key_move_backward = event.state == ElementState::Pressed;
                }
                Self::KEY_MOVE_LEFT => {
                    self.key_move_left = event.state == ElementState::Pressed;
                }
                Self::KEY_MOVE_RIGHT => {
                    self.key_move_right = event.state == ElementState::Pressed;
                }
                Self::KEY_MOVE_DOWN => {
                    self.key_move_down = event.state == ElementState::Pressed;
                }
                Self::KEY_MOVE_UP => {
                    self.key_move_up = event.state == ElementState::Pressed;
                }
                _ => (),
            },
            _ => (),
        }

        None
    }

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        let mut changed = false;

        if self.key_move_forward {
            self.camera_change.position.x -= 1.0 * delta_time as f32;
            changed = true;
        }

        if self.key_move_backward {
            self.camera_change.position.x += 1.0 * delta_time as f32;
            changed = true;
        }

        if self.key_move_left {
            self.camera_change.position.z -= 1.0 * delta_time as f32;
            changed = true;
        }

        if self.key_move_right {
            self.camera_change.position.z += 1.0 * delta_time as f32;
            changed = true;
        }

        if self.key_move_down {
            self.camera_change.position.y -= 1.0 * delta_time as f32;
            changed = true;
        }

        if self.key_move_up {
            self.camera_change.position.y += 1.0 * delta_time as f32;
            changed = true;
        }

        if changed {
            Some(vec![WorldChange::UpdateCamera(self.camera_change.clone())])
        } else {
            None
        }
    }
}
