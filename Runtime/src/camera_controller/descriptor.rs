use crate::camera_controller::{CameraControllerMovementType, CameraControllerRotationType};
use crate::resources::CameraDescriptor;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerDescriptor {
    /// Controls how the camera moves.
    pub movement_type: CameraControllerMovementType,
    /// Controls how the camera rotates.
    pub rotation_type: CameraControllerRotationType,
    /// The actual camera descriptor that is spawned and handles rendering.
    pub camera_descriptor: CameraDescriptor,
}
