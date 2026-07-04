use crate::{
    CameraControllerAxisInputMode, CameraControllerButtonInputMode, CameraControllerMouseInputMode,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CameraControllerRotationType {
    Free {
        axis_input: Option<CameraControllerAxisInputMode>,
        button_input: Option<CameraControllerButtonInputMode>,
        mouse_input: Option<CameraControllerMouseInputMode>,
        axis_dead_zone: f64,
    },
    Locked,
}
