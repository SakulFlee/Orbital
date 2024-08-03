use super::PositionChange;

#[derive(Debug, Default)]
pub struct CameraChange {
    pub target: &'static str,
    pub position: Option<PositionChange>,
    pub pitch: Option<f32>,
    pub yaw: Option<f32>,
}

impl CameraChange {
    pub fn does_change_something(&self) -> bool {
        self.position.is_some() || self.pitch.is_some() || self.yaw.is_some()
    }
}
