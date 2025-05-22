use crate::element::{CameraEvent, ModelEvent};

#[derive(Debug)]
pub enum PhysicsEvent {
    Model(ModelEvent),
    Camera(CameraEvent),
    Clear,
}
