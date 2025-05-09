use std::f32::consts::FRAC_PI_2;

use cgmath::{InnerSpace, Point3, Vector3};

use super::{CameraTransform, Mode};

#[derive(Debug, Clone, PartialEq)]
pub struct CameraDescriptor {
    pub label: String,
    pub is_active: bool,
    pub position: Point3<f32>,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
    pub global_gamma: f32,
}

impl CameraDescriptor {
    pub const DEFAULT_NAME: &'static str = "Default";
    pub const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

    pub fn apply_change(&mut self, change: CameraTransform) {
        if let Some(mode) = change.pitch {
            match mode {
                Mode::Overwrite(pitch) => self.pitch = pitch,
                Mode::Offset(pitch)
                | Mode::OffsetViewAligned(pitch)
                | Mode::OffsetViewAlignedWithY(pitch) => self.pitch += pitch,
            }

            if self.pitch < -Self::SAFE_FRAC_PI_2 {
                self.pitch = -Self::SAFE_FRAC_PI_2;
            } else if self.pitch > Self::SAFE_FRAC_PI_2 {
                self.pitch = Self::SAFE_FRAC_PI_2;
            }
        }

        if let Some(mode) = change.yaw {
            match mode {
                Mode::Overwrite(yaw) => self.yaw = yaw,
                Mode::Offset(yaw)
                | Mode::OffsetViewAligned(yaw)
                | Mode::OffsetViewAlignedWithY(yaw) => self.yaw += yaw,
            }
        }

        if let Some(mode) = change.position {
            match mode {
                Mode::Overwrite(position) => {
                    self.position = Point3 {
                        x: position.x,
                        y: position.y,
                        z: position.z,
                    };
                }
                Mode::Offset(position) => {
                    self.position += position;
                }
                Mode::OffsetViewAligned(position) => {
                    let (yaw_sin, yaw_cos) = self.yaw.sin_cos();

                    // Find alignment unit vectors
                    let unit_forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
                    let unit_right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();

                    // Apply offsets
                    self.position += unit_forward * position.x;
                    self.position += unit_right * position.z;
                    self.position.y += position.y;
                }
                Mode::OffsetViewAlignedWithY(position) => {
                    let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
                    let (pitch_sin, pitch_cos) = self.pitch.sin_cos();

                    // Find alignment unit vectors
                    let unit_forward =
                        Vector3::new(yaw_cos * pitch_cos, pitch_sin, yaw_sin * pitch_cos)
                            .normalize();
                    let unit_right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
                    let unit_up = unit_right.cross(unit_forward).normalize();

                    // Apply offsets
                    self.position += unit_forward * position.x;
                    self.position += unit_right * position.z;
                    self.position += unit_up * position.y;
                }
            }
        }
    }
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            label: Self::DEFAULT_NAME.into(),
            is_active: false,
            position: Point3::new(0.0, 0.0, 0.0),
            yaw: 0f32,
            pitch: 0f32,
            aspect: 16.0 / 9.0,
            fovy: 45.0,
            near: 0.1,
            far: 10000.0,
            global_gamma: 2.2,
        }
    }
}
