use crate::resources::{CameraDescriptor, CameraTransform, Mode, ModelDescriptor, Transform};

#[derive(Debug)]
pub enum CameraChange {
    Spawn(CameraDescriptor),
    Despawn(String),
    Activate(String),
    Deactivate(String),
    Transform(String, CameraTransform),
}
