use orbital::{
    game::{Element, ElementRegistration, WorldChange},
    loader::{GLTFLoader, GLTFWorkerMode},
    ulid::Ulid,
};

#[derive(Debug)]
pub struct DamagedHelmet;

impl Element for DamagedHelmet {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        const FILE_NAME: &str = "Assets/Models/DamagedHelmet.glb";

        ElementRegistration {
            world_changes: Some(vec![WorldChange::EnqueueLoader(Box::new(GLTFLoader::new(
                FILE_NAME,
                GLTFWorkerMode::LoadEverything,
            )))]),
            ..Default::default()
        }
    }
}

