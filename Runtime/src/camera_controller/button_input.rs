use crate::app::input::InputButton;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerButtonInputMode {
    /// Buttons to listen to.
    pub input_type: Vec<InputButton>, // TODO: use ButtonAxis
    /// Button multiplier.
    pub sensitivity: f32,
}
