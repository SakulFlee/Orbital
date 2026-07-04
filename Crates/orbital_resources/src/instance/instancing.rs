use super::InstanceDescriptor;

#[derive(Debug, Clone)]
pub enum Instancing {
    Single(InstanceDescriptor),
    Multiple(Vec<InstanceDescriptor>),
}
