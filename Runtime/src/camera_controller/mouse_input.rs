use crate::camera_controller::CameraControllerMouseInputType;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerMouseInputMode {
    /// How to handle mouse inputs.
    pub input_type: CameraControllerMouseInputType,
    /// Sensitivities for mouse inputs and multiplier.
    pub sensitivity: f32,
    /// If true, the cursor will be grabbed when the mouse is focused in the window.
    /// If false, the cursor remains unchanged.
    pub grab_cursor: bool,
    /// If true, the cursor will be hidden when the mouse is focused in the window.
    /// If false, the cursor remains unchanged.
    pub hide_cursor: bool,
}
