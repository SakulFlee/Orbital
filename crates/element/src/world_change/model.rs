use camera::Mode;
use model::{ModelDescriptor, Transform};

#[derive(Debug)]
pub enum ModelChange {
    Spawn(ModelDescriptor),
    Despawn(String),
    Transform(String, Mode<Transform>),
    TransformInstance(String, Mode<Transform>, usize),
    AddInstance(String, Transform),
    RemoveInstance(String, usize),
}
