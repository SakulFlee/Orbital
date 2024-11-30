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
        if self
            .position
            .as_ref()
            .is_some_and(|position| match position {
                Mode::Overwrite(v) => {
                    v.x.abs() >= 0.001 || v.y.abs() >= 0.001 || v.z.abs() >= 0.001
                }
                Mode::Offset(v) => v.x.abs() >= 0.001 || v.y.abs() >= 0.001 || v.z.abs() >= 0.0001,
                Mode::OffsetViewAligned(v) => {
                    v.x.abs() >= 0.001 || v.y.abs() >= 0.001 || v.z.abs() >= 0.001
                }
            })
        {
            return true;
        }

        if self.yaw.as_ref().is_some_and(|yaw| match yaw {
            Mode::Overwrite(v) => v.abs() >= 0.0001,
            Mode::Offset(v) => v.abs() >= 0.0001,
            Mode::OffsetViewAligned(v) => v.abs() >= 0.0001,
        }) {
            return true;
        }

        if self.pitch.as_ref().is_some_and(|pitch| match pitch {
            Mode::Overwrite(v) => v.abs() >= 0.0001,
            Mode::Offset(v) => v.abs() >= 0.0001,
            Mode::OffsetViewAligned(v) => v.abs() >= 0.0001,
        }) {
            return true;
        }

        false
    }
}
