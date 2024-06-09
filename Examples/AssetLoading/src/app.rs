use akimo_runtime::{
    cgmath::{Deg, Quaternion, Rotation3, Vector3},
    resources::{ImportDescriptor, InstanceDescriptor, Instancing, ModelDescriptor},
    runtime::App,
    server::RenderServer,
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
};

pub struct RenderServerTriangleApp {
    render_server: RenderServer,
}

impl App for RenderServerTriangleApp {
    fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
    where
        Self: Sized,
    {
        let mut render_server = RenderServer::new(config.format, device, queue);

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

        Self { render_server }
    }

    fn update(&mut self) {}

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        self.render_server.render(view, device, queue);
    }
}
