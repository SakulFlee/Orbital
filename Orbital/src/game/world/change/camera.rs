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
        self.position.is_some() || self.pitch.is_some() || self.yaw.is_some()
    }
}
