use orbital::{
    cgmath::Vector3,
    game::{Element, ElementRegistration, WorldChange},
    loader::{GLTFLoader, GLTFWorkerMode},
    transform::Transform,
};

#[derive(Debug)]
pub struct DamagedHelmet;

impl DamagedHelmet {
    const FILE_NAME: &'static str = "Assets/Models/DamagedHelmet.glb";
}

impl Element for DamagedHelmet {
    fn on_registration(&mut self) -> ElementRegistration {
        ElementRegistration::new(Self::FILE_NAME).with_initial_world_change(
            WorldChange::EnqueueLoader(Box::new(GLTFLoader::new(
                Self::FILE_NAME,
                GLTFWorkerMode::LoadEverything,
                Some(Transform {
                    position: Vector3::new(0.0, 0.0, 5.0),
                    ..Default::default()
                }),
            ))),
        )
    }

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        Some(vec![WorldChange::ApplyTransformModel(
            "mesh_helmet_LP_13930damagedHelmet".into(),
            Transform::only_position(Vector3::new(0.0, 0.0, 1.0 * delta_time as f32)),
        )])
    }
}
