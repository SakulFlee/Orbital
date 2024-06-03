use akimo_runtime::{
    resources::{ImportDescriptor, ModelDescriptor},
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
        let mut render_server = RenderServer::new(config.format);

        render_server.spawn_model(ModelDescriptor::FromGLTF(
            "Assets/Models/Cube.glb",
            ImportDescriptor::Index(0),
            ImportDescriptor::Index(0),
        ));

        Self { render_server }
    }

    fn update(&mut self) {}

    fn render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
        self.render_server.render(view, device, queue);
    }
}
