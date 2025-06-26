use crate::camera_controller::ButtonAxis;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerButtonInputMode {
    /// Buttons to listen to.
    pub input_type: Vec<ButtonAxis>,
    /// Button multiplier.
    pub sensitivity: f32,
}
