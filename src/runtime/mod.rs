use crate::{
    context::Context, error::RuntimeError, gpu_backend::GPUBackend, logging::init_logger,
    surface_wrapper::SurfaceWrapper, window_wrapper::WindowWrapper,
};

use log::info;
use winit::event::{Event, WindowEvent};

mod settings;
pub use settings::*;

pub struct Runtime;

impl Runtime {
    pub async fn liftoff(settings: RuntimeSettings) -> Result<(), RuntimeError> {
        init_logger();
        info!("Akimo-Project: Engine");
        info!("(C) SakulFlee 2024");

        let window_wrapper = WindowWrapper::new(&settings.name, settings.size);
        let window = window_wrapper.window();
        let mut surface = SurfaceWrapper::new();
        let context = Context::init(&mut surface).await;

        info!("Staring event loop ...");
        let _ = window_wrapper
            .event_loop()
            .run(|event, target| match event {
                Event::Resumed => {
                    surface.resume(&context, window.clone());
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    _ => (),
                },
                _ => (),
            });

        Ok(())
    }
}
