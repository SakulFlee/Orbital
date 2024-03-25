use crate::{
    app::App, context::Context, error::RuntimeError, logging::init_logger,
    surface_wrapper::SurfaceWrapper, window_wrapper::WindowWrapper,
};

use log::{error, info};
use wgpu::TextureViewDescriptor;
use winit::event::{Event, WindowEvent};

mod settings;
pub use settings::*;

pub struct Runtime;

impl Runtime {
    pub fn liftoff<AppImpl: App>(settings: RuntimeSettings) -> Result<(), RuntimeError> {
        pollster::block_on(Self::liftoff_async::<AppImpl>(settings))
    }

    pub async fn liftoff_async<AppImpl: App>(
        settings: RuntimeSettings,
    ) -> Result<(), RuntimeError> {
        init_logger();
        info!("Akimo-Project: Engine");
        info!("(C) SakulFlee 2024");

        let window_wrapper = WindowWrapper::new(&settings.name, settings.size);
        let window = window_wrapper.window();
        let mut surface = SurfaceWrapper::new();
        let context = Context::init(&mut surface).await;

        let mut app: Option<AppImpl> = None;

        info!("Staring event loop ...");
        let _ = window_wrapper
            .event_loop()
            .run(|event, target| match event {
                Event::Resumed => {
                    info!("Resuming ...");
                    surface.resume(&context, window.clone());

                    if app.is_none() {
                        info!("Bootstrapping app ...");
                        app = Some(AppImpl::init(
                            surface.configuration(),
                            context.adapter(),
                            context.device(),
                            context.queue(),
                        ));
                    }
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
                        app.as_mut()
                            .expect("Redraw requested when app is none!")
                            .render(&view, context.device(), context.queue());

                        // Present the frame after rendering and inform the window about a redraw being needed
                        frame.present();
                        window.request_redraw();
                    }
                    _ => app.as_mut().expect("App gone").update(event),
                },
                _ => (),
            });

        Ok(())
    }
}
