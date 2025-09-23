use orbital::{
    async_trait::async_trait,
    cgmath::Vector3,
    element::{Element, ElementRegistration, Event, WorldEvent, LightEvent},
    resources::LightDescriptor,
};

#[derive(Debug)]
pub struct TestLights;

#[async_trait]
impl Element for TestLights {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new("test_lights").with_initial_events(vec![
            Event::World(WorldEvent::Light(LightEvent::Spawn(
                LightDescriptor::new_point(
                    "Point Light 1".to_string(),
                    Vector3::new(5.0, 5.0, 5.0),
                    Vector3::new(1.0, 1.0, 1.0),
                    10.0,
                ),
            ))),
            Event::World(WorldEvent::Light(LightEvent::Spawn(
                LightDescriptor::new_directional(
                    "Directional Light".to_string(),
                    Vector3::new(-1.0, -1.0, -1.0),
                    Vector3::new(1.0, 1.0, 1.0),
                    1.0,
                ),
            ))),
        ])
    }
}
