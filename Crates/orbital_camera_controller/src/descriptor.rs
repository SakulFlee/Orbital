use crate::{CameraControllerMovementType, CameraControllerRotationType};
use orbital_resources::CameraDescriptor;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraControllerDescriptor {
    pub movement_type: CameraControllerMovementType,
    pub rotation_type: CameraControllerRotationType,
    pub camera_descriptor: CameraDescriptor,
}
