use akimo_project::{
    app::App,
    error::RuntimeError,
    logging::*,
    runtime::{Runtime, RuntimeSettings},
};

pub struct EngineApp;

impl EngineApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl App for EngineApp {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        todo!()
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
        todo!()
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        todo!()
    }
}

fn main() -> Result<(), RuntimeError> {
    let engine_app = EngineApp::new();
    pollster::block_on(Runtime::liftoff(engine_app, RuntimeSettings::default()))
}
