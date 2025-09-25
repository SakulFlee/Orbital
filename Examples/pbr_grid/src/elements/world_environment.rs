use orbital::{
    element::{Element, ElementRegistration, EnvironmentEvent, Event, WorldEvent},
    resources::WorldEnvironmentDescriptor,
};

#[derive(Debug)]
pub struct WorldEnvironment;

impl WorldEnvironment {}

impl Element for WorldEnvironment {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new("world_environment").with_initial_event(Event::World(
            WorldEvent::Environment(EnvironmentEvent::Change {
                descriptor: WorldEnvironmentDescriptor::FromFile {
                    cube_face_size: 2048,
                    path: "Assets/WorldEnvironments/PhotoStudio.hdr".to_string(),
                    sampling_type: WorldEnvironmentDescriptor::DEFAULT_SAMPLING_TYPE,
                    custom_specular_mip_level_count: None,
                },
            }),
        ))
    }
}
