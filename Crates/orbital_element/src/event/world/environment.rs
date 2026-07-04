use orbital_resources::WorldEnvironmentDescriptor;

#[derive(Debug)]
pub enum EnvironmentEvent {
    Change {
        descriptor: WorldEnvironmentDescriptor,
    },
}
