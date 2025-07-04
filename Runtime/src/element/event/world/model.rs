use crate::resources::{Mode, ModelDescriptor, Transform};

#[derive(Debug)]
pub enum ModelEvent {
    Spawn(ModelDescriptor),
    Despawn(String),
    Transform(String, Mode<Transform>),
    TransformInstance(String, Mode<Transform>, usize),
    AddInstance(String, Transform),
    RemoveInstance(String, usize),
}
