use orbital::{
    cgmath::Vector3,
    game::{Element, ElementRegistration, WorldChange},
    loader::{GLTFLoader, GLTFWorkerMode},
    resources::descriptors::LightDescriptor,
    ulid::Ulid,
};

#[derive(Debug)]
pub struct PBRSpheres;

impl Element for PBRSpheres {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        const FILE_NAME: &str = "Assets/Models/PBR_Spheres.glb";

        ElementRegistration {
            world_changes: Some(vec![
                WorldChange::EnqueueLoader(Box::new(GLTFLoader::new(
                    FILE_NAME,
                    GLTFWorkerMode::LoadEverything,
                ))),
                WorldChange::SpawnLight(LightDescriptor::PointLight {
                    position: Vector3::new(5.0, 0.0, 0.0),
                    color: Vector3::new(1.0, 1.0, 1.0),
                }),
                WorldChange::SpawnLight(LightDescriptor::PointLight {
                    position: Vector3::new(5.0, 5.0, 5.0),
                    color: Vector3::new(1.0, 0.0, 0.0),
                }),
                WorldChange::SpawnLight(LightDescriptor::PointLight {
                    position: Vector3::new(5.0, 5.0, -5.0),
                    color: Vector3::new(0.0, 1.0, 0.0),
                }),
                WorldChange::SpawnLight(LightDescriptor::PointLight {
                    position: Vector3::new(5.0, -5.0, -5.0),
                    color: Vector3::new(0.0, 0.0, 1.0),
                }),
            ]),
            ..Default::default()
        }
    }
}
