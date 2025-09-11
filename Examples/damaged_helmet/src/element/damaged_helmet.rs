use orbital::{
    async_trait::async_trait,
    element::{Element, ElementRegistration, Event, WorldEvent},
    importer::{ImportTask, gltf::GltfImport},
};

#[derive(Debug)]
pub struct DamagedHelmet;
impl DamagedHelmet {
    const FILE_NAME: &'static str = "Assets/Models/DamagedHelmet.glb";
}

#[async_trait]
impl Element for DamagedHelmet {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new(Self::FILE_NAME).with_initial_world_change(Event::World(
            WorldEvent::Import(ImportTask::Gltf {
                file_path: Self::FILE_NAME.to_string(),
                task: GltfImport::WholeFile,
            }),
        ))
    }
}
