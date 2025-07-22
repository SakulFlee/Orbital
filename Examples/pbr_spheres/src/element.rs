use orbital::element::{Element, ElementRegistration, Event, ModelEvent, WorldEvent};

#[derive(Debug)]
pub struct PBRSpheres;

impl PBRSpheres {
    const FILE_NAME: &'static str = "Assets/Models/PBR_Spheres.glb";
}

impl Element for PBRSpheres {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new(Self::FILE_NAME).with_initial_world_change(Event::World(WorldEvent::Model(ModelEvent::)))
    }
}
