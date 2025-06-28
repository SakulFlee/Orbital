use crate::resources::WorldEnvironmentDescriptor;

#[derive(Debug)]
pub enum EnvironmentEvent {
    Change {
        descriptor: WorldEnvironmentDescriptor,
        enable_ibl: bool,
    },
}
