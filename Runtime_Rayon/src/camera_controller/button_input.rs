use crate::camera_controller::ButtonAxis;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerButtonInputMode {
    /// Buttons to listen to.
    pub button_axis: Vec<ButtonAxis>,
    /// Button multiplier.
    pub sensitivity: f32,
}
