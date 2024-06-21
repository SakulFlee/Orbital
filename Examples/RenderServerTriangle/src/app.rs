use akimo_runtime::{
    app::App,
    cgmath::{Vector2, Vector3},
    renderer::{Renderer, TestRenderer},
    resources::{
        descriptors::{
            InstanceDescriptor, Instancing, MaterialDescriptor, MeshDescriptor, ModelDescriptor,
            TextureDescriptor,
        },
        realizations::{Model, Vertex},
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
        let vertices = vec![
            Vertex {
                position_coordinates: Vector3::new(-1.0, -1.0, 0.0),
                texture_coordinates: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position_coordinates: Vector3::new(1.0, -1.0, 0.0),
                texture_coordinates: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position_coordinates: Vector3::new(0.0, 1.0, 0.0),
                texture_coordinates: Vector2::new(0.0, 0.0),
            },
        ];
        let indices = vec![0, 1, 2];

        let renderer = TestRenderer::new(config.format, (1280, 720).into(), device, queue);

        let model = Model::from_descriptor(
            &ModelDescriptor::FromDescriptors(
                MeshDescriptor { vertices, indices },
                MaterialDescriptor::PBRCustomShader(
                    TextureDescriptor::EMPTY,
                    include_str!("rgb.wgsl"),
                ),
                Instancing::Single(InstanceDescriptor::default()),
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

    fn on_render(&mut self, target_view: &TextureView, device: &Device, queue: &Queue) {
        self.renderer
            .render(target_view, device, queue, &self.models);
    }
}
