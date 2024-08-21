use orbital::{
    cgmath::{Deg, Quaternion, Rotation3, Vector3},
    game::{Element, ElementRegistration, WorldChange},
    resources::descriptors::{
        ImportDescriptor, InstanceDescriptor, Instancing, LightDescriptor, ModelDescriptor,
    },
    ulid::Ulid,
};

#[derive(Debug)]
pub struct DamagedHelmet;

impl Element for DamagedHelmet {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        // We can directly supply our ModelDescriptor during registration.
        // Alternatively, we could queue a WorldChange::SpawnModel(Owned).
        ElementRegistration {
            models: Some(vec![ModelDescriptor::FromGLTF(
                "Assets/Models/DamagedHelmet.glb",
                ImportDescriptor::Index(0),
                ImportDescriptor::Index(0),
                Instancing::Single(InstanceDescriptor {
                    position: Vector3::new(0.0, 0.0, -5.0),
                    rotation: Quaternion::from_angle_y(Deg(90.0)),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                }),
            )]),
            world_changes: Some(vec![WorldChange::SpawnLight(LightDescriptor::PointLight {
                position: Vector3::new(0.0, 0.0, 0.0),
                color: Vector3::new(1.0, 1.0, 1.0),
            })]),
            ..Default::default()
        }
    }
}
