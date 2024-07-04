use akimo_runtime::{
    cgmath::{Deg, Quaternion, Rotation3, Vector3},
    game::{Element, ElementRegistration},
    resources::descriptors::{ImportDescriptor, InstanceDescriptor, Instancing, ModelDescriptor},
    ulid::Ulid,
};

pub struct Cubes;

impl Element for Cubes {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        // We can directly supply our ModelDescriptor during registration.
        // Alternatively, we could queue a WorldChange::SpawnModel(Owned).
        ElementRegistration {
            models: Some(vec![ModelDescriptor::FromGLTF(
                "Assets/Models/Cube.glb",
                ImportDescriptor::Index(0),
                ImportDescriptor::Index(0),
                Instancing::Multiple(vec![
                    // Default
                    InstanceDescriptor::default(),
                    // Rotation
                    InstanceDescriptor {
                        position: Vector3::new(0.0, -1.0, -1.0),
                        rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(45.0)),
                        ..Default::default()
                    },
                    InstanceDescriptor {
                        position: Vector3::new(0.0, -1.0, 0.0),
                        rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(45.0)),
                        ..Default::default()
                    },
                    InstanceDescriptor {
                        position: Vector3::new(0.0, -1.0, 1.0),
                        rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(45.0)),
                        ..Default::default()
                    },
                    // Scale test
                    InstanceDescriptor {
                        position: Vector3::new(0.0, 1.0, -1.0),
                        scale: Vector3::new(2.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    InstanceDescriptor {
                        position: Vector3::new(0.0, 1.0, 0.0),
                        scale: Vector3::new(1.0, 2.0, 1.0),
                        ..Default::default()
                    },
                    InstanceDescriptor {
                        position: Vector3::new(0.0, 1.0, 1.0),
                        scale: Vector3::new(1.0, 1.0, 2.0),
                        ..Default::default()
                    },
                ]),
            )]),
            ..Default::default()
        }
    }
}
