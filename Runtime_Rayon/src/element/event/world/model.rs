use crate::resources::{Mode, ModelDescriptor, Transform};

#[derive(Debug)]
pub enum ModelEvent {
    Spawn(ModelDescriptor),
    Despawn(String),
    Transform(String, Mode<Transform>),
    TransformInstance(String, Mode<Transform>, String), // ULID as string
    AddInstance(String, Transform),
    RemoveInstance(String, String), // ULID as string
}
