use crate::resources::{CameraDescriptor, CameraTransform};

#[derive(Debug)]
pub enum CameraEvent {
    Spawn(CameraDescriptor),
    Despawn(String),
    Target(String),
    Transform(CameraTransform),
}
