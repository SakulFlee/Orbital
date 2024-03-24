use crate::{
    app::App, context::Context, error::RuntimeError, logging::init_logger,
    surface_wrapper::SurfaceWrapper, window_wrapper::WindowWrapper,
};

use log::info;
use wgpu::TextureViewDescriptor;
use winit::event::{Event, WindowEvent};

mod settings;
pub use settings::*;

pub struct Runtime;

impl Runtime {
    pub async fn liftoff(mut app: impl App, settings: RuntimeSettings) -> Result<(), RuntimeError> {
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
                    WindowEvent::RedrawRequested => {
                        // Get next frame to render onto
                        let frame = surface.acquire_next_frame(&context);
                        let view = frame.texture.create_view(&TextureViewDescriptor {
                            format: Some(surface.configuration().view_formats[0]),
                            ..TextureViewDescriptor::default()
                        });

                        // Render!
                        app.render(&view, context.device(), context.queue());

                        // Present the frame after rendering and inform the window about a redraw being needed
                        frame.present();
                        window.request_redraw();
                    }
                    _ => (),
                },
                _ => (),
            });

        Ok(())
    }
}
