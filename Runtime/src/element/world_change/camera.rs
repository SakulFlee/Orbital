use crate::resources::{CameraDescriptor, CameraTransform};

#[derive(Debug)]
pub enum CameraChange {
    Spawn(CameraDescriptor),
    Despawn(String),
    Target(String),
    Transform(String, CameraTransform),
}
