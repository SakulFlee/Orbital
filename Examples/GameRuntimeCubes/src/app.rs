use akimo_runtime::{
    cgmath::{Deg, Quaternion, Rotation3, Vector3},
    game::{Game, World, WorldChange},
    resources::descriptors::{ImportDescriptor, InstanceDescriptor, Instancing, ModelDescriptor},
};

pub struct ExampleGame;

impl Game for ExampleGame {
    fn init() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn on_update(&mut self, _delta_time: f64, world: &mut World)
    where
        Self: Sized,
    {
        if world.composition.is_empty() {
            world.queue_world_change(WorldChange::SpawnModels(vec![ModelDescriptor::FromGLTF(
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
            )]));
        }
    }
}
