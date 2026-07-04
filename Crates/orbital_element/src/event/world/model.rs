use orbital_resources::{Mode, ModelDescriptor, Transform};

#[derive(Debug)]
pub enum ModelEvent {
    Spawn(ModelDescriptor),
    Despawn(String),
    Transform(String, Mode<Transform>),
    TransformInstance(String, Mode<Transform>, String),
    AddInstance(String, Transform),
    RemoveInstance(String, String),
}
