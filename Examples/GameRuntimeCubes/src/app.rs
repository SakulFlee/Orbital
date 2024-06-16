use akimo_runtime::{
    cgmath::{Deg, Quaternion, Rotation3, Vector3},
    game::Game,
    resources::descriptors::{ImportDescriptor, InstanceDescriptor, Instancing, ModelDescriptor},
    server::RenderServer,
};

pub struct ExampleGame {
    initialized: bool,
}

impl Game for ExampleGame {
    fn init() -> Self
    where
        Self: Sized,
    {
        Self { initialized: false }
    }

    fn cycle(&mut self, _delta_time: f64, render_server: &mut RenderServer)
    where
        Self: Sized,
    {
        if !self.initialized {
            self.initialized = true;

            render_server.spawn_model(ModelDescriptor::FromGLTF(
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
            ));
        }
    }
}
