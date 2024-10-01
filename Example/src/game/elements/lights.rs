use orbital::{
    cgmath::Vector3,
    game::{Element, ElementRegistration, WorldChange},
    resources::{descriptors::LightDescriptor, realizations::PointLight},
};

#[derive(Debug)]
pub struct Lights;

impl Element for Lights {
    fn on_registration(&mut self) -> ElementRegistration {
        ElementRegistration::new("Lights").with_initial_world_changes(vec![
            WorldChange::SpawnLight(LightDescriptor::PointLight(PointLight {
                label: "Scene Lights".into(),
                position: Vector3::new(0.0, 0.0, 5.0),
                color: Vector3::new(1.0, 1.0, 1.0),
            })),
            WorldChange::SpawnLight(LightDescriptor::PointLight(PointLight {
                label: "Scene Lights".into(),
                position: Vector3::new(0.0, 0.0, 10.0),
                color: Vector3::new(1.0, 0.0, 0.0),
            })),
            WorldChange::SpawnLight(LightDescriptor::PointLight(PointLight {
                position: Vector3::new(0.0, 0.0, 15.0),
                label: "Scene Lights".into(),
                color: Vector3::new(0.0, 1.0, 0.0),
            })),
            WorldChange::SpawnLight(LightDescriptor::PointLight(PointLight {
                label: "Scene Lights".into(),
                position: Vector3::new(0.0, 0.0, 20.0),
                color: Vector3::new(0.0, 0.0, 1.0),
            })),
            WorldChange::SpawnLight(LightDescriptor::PointLight(PointLight {
                label: "Scene Lights".into(),
                position: Vector3::new(0.0, 0.0, 25.0),
                color: Vector3::new(1.0, 1.0, 1.0),
            })),
        ])
    }
}
