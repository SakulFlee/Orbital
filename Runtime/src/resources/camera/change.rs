use cgmath::Vector3;

use super::Mode;

#[derive(Debug, Default)]
pub struct CameraTransform {
    /// Label of the camera this change should be applied to.
    pub label: String,
    /// If `Some`, will change the camera's position.
    /// If `None`, will be ignored.
    pub position: Option<Mode<Vector3<f32>>>,
    /// If `Some`, will change the camera's pitch.
    /// If `None`, will be ignored.
    pub pitch: Option<Mode<f32>>,
    /// If `Some`, will change the camera's yaw.
    /// If `None`, will be ignored.
    pub yaw: Option<Mode<f32>>,
}

impl CameraTransform {
    pub fn is_introducing_change(&self) -> bool {
        if self
            .position
            .as_ref()
            .is_some_and(|position| match position {
                Mode::Overwrite(v)
                | Mode::Offset(v)
                | Mode::OffsetViewAligned(v)
                | Mode::OffsetViewAlignedWithY(v) => {
                    v.x.abs() >= 0.001 || v.y.abs() >= 0.001 || v.z.abs() >= 0.001
                }
            })
        {
            return true;
        }

        if self.yaw.as_ref().is_some_and(|yaw| match yaw {
            Mode::Overwrite(v)
            | Mode::Offset(v)
            | Mode::OffsetViewAligned(v)
            | Mode::OffsetViewAlignedWithY(v) => v.abs() >= 0.0001,
        }) {
            return true;
        }

        if self.pitch.as_ref().is_some_and(|pitch| match pitch {
            Mode::Overwrite(v)
            | Mode::Offset(v)
            | Mode::OffsetViewAligned(v)
            | Mode::OffsetViewAlignedWithY(v) => v.abs() >= 0.0001,
        }) {
            return true;
        }

        false
    }
}
