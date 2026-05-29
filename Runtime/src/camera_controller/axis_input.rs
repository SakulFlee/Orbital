use crate::app::input::InputAxis;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerAxisInputMode {
    /// Axis to listen to.
    pub axis: Vec<InputAxis>,
    /// Axis sensitivity and multiplier.
    pub sensitivity: f32,
}
