use crate::app::input::InputAxis;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerAxisInputMode {
    /// Axis to listen to.
    pub input_type: Vec<InputAxis>,
    /// Axis sensitivity and multiplier.
    pub sensitivity: f32,
}
