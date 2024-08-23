use orbital::{
    cgmath::Vector3,
    game::{Element, ElementRegistration, WorldChange},
    resources::descriptors::{
        ImportDescriptor, InstanceDescriptor, Instancing, LightDescriptor, ModelDescriptor,
    },
    ulid::Ulid,
};

#[derive(Debug)]
pub struct PBRSpheres;

impl Element for PBRSpheres {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        ElementRegistration {
            models: Some(vec![ModelDescriptor::FromGLTF(
                "Assets/Models/PBR_Spheres.glb",
                ImportDescriptor::Index(0),
                ImportDescriptor::Index(0),
                Instancing::Single(InstanceDescriptor::default()),
            )]),
            world_changes: Some(vec![
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
