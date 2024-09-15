use orbital::{
    cgmath::Vector3,
    game::{Element, ElementRegistration, WorldChange},
    loader::{GLTFLoader, GLTFWorkerMode},
    transform::Transform,
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
                Some(Transform {
                    position: Vector3::new(0.0, 0.0, 5.0),
                    ..Default::default()
                }),
            )))]),
            ..Default::default()
        }
    }

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        Some(vec![WorldChange::ApplyTransformModel(
            "mesh_helmet_LP_13930damagedHelmet".into(),
            Transform::only_position(Vector3::new(0.0, 0.0, 1.0 * delta_time as f32)),
        )])
    }
}
