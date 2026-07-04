use crate::ButtonAxis;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerButtonInputMode {
    pub button_axis: Vec<ButtonAxis>,
    pub sensitivity: f32,
}
