use std::f32::consts::FRAC_PI_2;

use cgmath::{InnerSpace, Point3, Vector3};

use crate::game::{CameraChange, PositionMode};

#[derive(Debug, Clone, PartialEq)]
pub struct CameraDescriptor {
    pub identifier: String,
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

impl CameraDescriptor {
    pub const DEFAULT_NAME: &'static str = "Default";

    pub fn apply_change(&mut self, change: CameraChange) {
        if let Some(pitch) = change.pitch {
            self.pitch = pitch;

            if self.pitch < -FRAC_PI_2 { // TODO: Test
                self.pitch = -FRAC_PI_2;
            } else if self.pitch > FRAC_PI_2 {
                self.pitch = FRAC_PI_2;
            }
        }

        if let Some(yaw) = change.yaw {
            self.yaw = yaw;
        }

        if let Some(position) = change.position {
            match position.mode {
                PositionMode::Overwrite => {
                    self.position = Point3 {
                        x: position.position.x,
                        y: position.position.y,
                        z: position.position.z,
                    };
                }
                PositionMode::Offset => {
                    self.position += position.position;
                }
                PositionMode::OffsetViewAligned => {
                    let (yaw_sin, yaw_cos) = self.yaw.sin_cos();

                    // Find alignment unit vectors
                    let unit_forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
                    let unit_right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

                    // Apply offsets
                    self.position += unit_forward * position.position.x;
                    self.position += unit_right * position.position.z;
                    self.position.y += position.position.y;
                }
            }
        }
    }
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            identifier: Self::DEFAULT_NAME.into(),
            position: Point3::new(-1.0, 0.0, 0.0),
            yaw: 0f32,
            pitch: 0f32,
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            near: 0.1,
            far: 10000.0,
        }
    }
}
