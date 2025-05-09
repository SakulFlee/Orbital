use camera::{CameraDescriptor, CameraTransform, Mode};
use model::{ModelDescriptor, Transform};

#[derive(Debug)]
pub enum CameraChange {
    Spawn(CameraDescriptor),
    Despawn(String),
    Activate(String),
    Deactivate(String),
    Transform(String, CameraTransform),
}
