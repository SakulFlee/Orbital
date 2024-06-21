use akimo_runtime::{
    app::App,
    cgmath::{Deg, Quaternion, Rotation3, Vector3},
    renderer::{Renderer, TestRenderer},
    resources::{
        descriptors::{ImportDescriptor, InstanceDescriptor, Instancing, ModelDescriptor},
        realizations::Model,
    },
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
};

pub struct RenderServerTriangleApp {
    renderer: TestRenderer,
    models: Vec<Model>,
}

impl App for RenderServerTriangleApp {
    fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
    where
        Self: Sized,
    {
        let renderer = TestRenderer::new(
            config.format,
            (config.width, config.height).into(),
            device,
            queue,
        );

        let model = Model::from_descriptor(
            &ModelDescriptor::FromGLTF(
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
            ),
            device,
            queue,
        )
        .expect("Model loading failed");

        Self {
            renderer,
            models: vec![model],
        }
    }

    fn on_update(&mut self) {}

    fn on_render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        self.renderer.render(view, device, queue, &self.models);
    }
}
