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

        /// For most controllers/gamepads, something around 0.1 should suffice.
        /// This value depends highly on your controller and how much e.g. "stick drift" you have.
        /// On platforms that manage controller dead zones (e.g. consoles) this might not be required
        /// and can simply be set to 0.0! However, setting this to anything but zero on managed platforms
        /// shouldn't interfere either.
        axis_dead_zone: f64,
    },
    /// A camera controller that is locked and will not be rotated automatically.
    /// A locked camera can still be interacted with and manually rotated!
    Locked,
}
