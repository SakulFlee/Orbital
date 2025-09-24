use orbital::{
    element::{Element, ElementRegistration, Event, WorldEvent},
    importer::{ImportTask, gltf::GltfImport},
};

#[derive(Debug)]
pub struct PBRSpheres;

impl PBRSpheres {
    const FILE_NAME: &'static str = "Assets/Models/InstancingTest.glb";
}

impl Element for PBRSpheres {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new(Self::FILE_NAME).with_initial_event(Event::World(
            WorldEvent::Import(ImportTask::Gltf {
                file_path: Self::FILE_NAME.into(),
                task: GltfImport::WholeFile,
            }),
        ))
    }
}
