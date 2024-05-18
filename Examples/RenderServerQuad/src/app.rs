use akimo_runtime::{
    nalgebra::Vector3,
    resources::{MaterialDescriptor, MeshDescriptor, ModelDescriptor, Vertex},
    runtime::App,
    server::RenderServer,
    wgpu::{Device, Queue, SurfaceConfiguration, TextureView},
};

pub struct RenderServerTriangleApp {
    render_server: RenderServer,
}

impl App for RenderServerTriangleApp {
    fn init(config: &SurfaceConfiguration, _device: &Device, _queue: &Queue) -> Self
    where
        Self: Sized,
    {
        let vertices = vec![
            // Bottom Left
            Vertex {
                position_coordinates: Vector3::new(-0.5, -0.5, 0.0),
            },
            // Bottom Right
            Vertex {
                position_coordinates: Vector3::new(0.5, -0.5, 0.0),
            },
            // Top Left
            Vertex {
                position_coordinates: Vector3::new(-0.5, 0.5, 0.0),
            },
            // Top Right
            Vertex {
                position_coordinates: Vector3::new(0.5, 0.5, 0.0),
            },
        ];
        let indices = vec![
            0, 1, 2, // Bottom Left Triangle
            1, 2, 3, // Top Right Triangle
        ];

        let mut render_server = RenderServer::new(config.format);

        render_server.spawn_model(ModelDescriptor {
            mesh_descriptor: MeshDescriptor { vertices, indices },
            material_descriptor: MaterialDescriptor::NoImports,
        });

        Self { render_server }
    }

    fn update(&mut self) {}

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        self.render_server.render(view, device, queue);
    }
}
