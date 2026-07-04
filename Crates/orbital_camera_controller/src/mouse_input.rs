use crate::CameraControllerMouseInputType;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerMouseInputMode {
    pub input_type: CameraControllerMouseInputType,
    pub sensitivity: f32,
    pub grab_cursor: bool,
    pub hide_cursor: bool,
}
