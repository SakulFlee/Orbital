use cgmath::Vector3;

use super::Mode;

#[derive(Debug, Default)]
pub struct CameraChange {
    pub target: &'static str,
    pub position: Option<Mode<Vector3<f32>>>,
    pub pitch: Option<Mode<f32>>,
    pub yaw: Option<Mode<f32>>,
}

impl CameraChange {
    pub fn does_change_something(&self) -> bool {
        if let Some(position) = &self.position {
            return match position {
                Mode::Overwrite(v) => {
                    v.x.abs() >= 0.001 || v.y.abs() >= 0.001 || v.z.abs() >= 0.001
                }
                Mode::Offset(v) => v.x.abs() >= 0.001 || v.y.abs() >= 0.001 || v.z.abs() >= 0.0001,
                Mode::OffsetViewAligned(v) => {
                    v.x.abs() >= 0.001 || v.y.abs() >= 0.001 || v.z.abs() >= 0.001
                }
            };
        }

        if let Some(yaw) = &self.yaw {
            return match yaw {
                Mode::Overwrite(v) => v.abs() >= 0.0001,
                Mode::Offset(v) => v.abs() >= 0.0001,
                Mode::OffsetViewAligned(v) => v.abs() >= 0.0001,
            };
        }

        if let Some(pitch) = &self.pitch {
            return match pitch {
                Mode::Overwrite(v) => v.abs() >= 0.0001,
                Mode::Offset(v) => v.abs() >= 0.0001,
                Mode::OffsetViewAligned(v) => v.abs() >= 0.0001,
            };
        }

        false
    }
}
