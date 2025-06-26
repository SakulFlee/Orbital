use crate::camera_controller::{
    CameraControllerAxisInputMode, CameraControllerButtonInputMode, CameraControllerMouseInputMode,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CameraControllerRotationType {
    /// A free camera controller that can move around freely without any constraints.
    Free {
        /// Controls which axis can rotate the camera.
        axis_input: Option<CameraControllerAxisInputMode>,
        /// Controls which buttons can move the camera.
        button_input: Option<CameraControllerButtonInputMode>,
        /// Controls mouse behavior
        mouse_input: Option<CameraControllerMouseInputMode>,
    },
    /// A camera controller that is locked and will not be rotated automatically.
    /// A locked camera can still be interacted with and manually rotated!
    Locked,
}
