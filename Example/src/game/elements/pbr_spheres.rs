use orbital::{
    game::{Element, ElementRegistration, WorldChange},
    loader::{GLTFLoader, GLTFWorkerMode},
    ulid::Ulid,
};

#[derive(Debug)]
pub struct PBRSpheres;

impl Element for PBRSpheres {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        const FILE_NAME: &str = "Assets/Models/PBR_Spheres.glb";

        ElementRegistration {
            world_changes: Some(vec![WorldChange::EnqueueLoader(Box::new(GLTFLoader::new(
                FILE_NAME,
                GLTFWorkerMode::LoadEverything,
                None,
            )))]),
            ..Default::default()
        }
    }
}
