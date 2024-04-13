use akimo_runtime::{
    nalgebra::Vector3,
    render_server::RenderServer,
    resources::{Mesh, Vertex},
    runtime::{App, Context},
    wgpu::{SurfaceConfiguration, TextureView},
};

pub struct RenderServerTriangleApp {
    render_server: RenderServer,
}

impl App for RenderServerTriangleApp {
    fn init(config: &SurfaceConfiguration, context: &Context) -> Self
    where
        Self: Sized,
    {
        let vertices = vec![
            Vertex {
                position_coordinates: Vector3::new(-1.0, -1.0, 0.0),
            },
            Vertex {
                position_coordinates: Vector3::new(1.0, -1.0, 0.0),
            },
            Vertex {
                position_coordinates: Vector3::new(0.0, 1.0, 0.0),
            },
        ];
        let indices = vec![0, 1, 2];

        let mesh = Mesh::from_vertex_index(context, vertices, indices);

        let mut render_server = RenderServer::new(context, config.format);
        render_server.add_mesh(mesh);

        Self { render_server }
    }

    fn resize(&mut self, _config: &SurfaceConfiguration, _context: &Context) {}

    fn update(&mut self) {}

    fn render(&mut self, view: &TextureView, context: &Context) {
        self.render_server.render(view, context);
    }
}
