use orbital::{
    world::{Element, ElementRegistration, WorldChange},
    loader::{GLTFLoader, GLTFWorkerMode},
};

#[derive(Debug)]
pub struct PBRSpheres;

impl PBRSpheres {
    const FILE_NAME: &'static str = "Assets/Models/PBR_Spheres.glb";
}

impl Element for PBRSpheres {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new(Self::FILE_NAME).with_initial_world_change(
            WorldChange::EnqueueLoader(Box::new(GLTFLoader::new(
                Self::FILE_NAME,
                GLTFWorkerMode::LoadEverything,
                None,
            ))),
        )
    }
}
