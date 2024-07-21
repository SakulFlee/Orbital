use cgmath::Point3;

use crate::{
    game::{Element, ElementRegistration, WorldChange},
    resources::descriptors::CameraDescriptor,
};

pub struct DebugTestCamera {
    incrementing: bool,
    camera_change: CameraDescriptor,
}

impl DebugTestCamera {
    pub const DEBUG_CAMERA_NAME: &'static str = "DEBUG";

    pub fn new() -> Self {
        Self {
            incrementing: true,
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

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        if self.incrementing {
            self.camera_change.position.x += 1.0 * delta_time as f32;
        } else {
            self.camera_change.position.x -= 1.0 * delta_time as f32;
        }

        if self.camera_change.position.x >= 5.0 {
            self.incrementing = false;
        }
        if self.camera_change.position.x <= 1.5 {
            self.incrementing = true;
        }

        Some(vec![WorldChange::UpdateCamera(self.camera_change.clone())])
    }
}
