use crate::{
    app::App, context::Context, error::RuntimeError, surface_wrapper::SurfaceWrapper,
    window_wrapper::WindowWrapper,
};

use log::{debug, info, warn};
use wgpu::TextureViewDescriptor;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

mod settings;
pub use settings::*;

pub struct Runtime;

impl Runtime {
    pub fn liftoff<AppImpl: App>(
        event_loop: EventLoop<()>,
        settings: RuntimeSettings,
    ) -> Result<(), RuntimeError> {
        debug!("LIFTOFF");
        pollster::block_on(Self::liftoff_async::<AppImpl>(event_loop, settings))
    }

    pub async fn liftoff_async<AppImpl: App>(
        event_loop: EventLoop<()>,
        settings: RuntimeSettings,
    ) -> Result<(), RuntimeError> {
        info!("Akimo-Project: Runtime");
        info!("(C) SakulFlee 2024");

        let window_wrapper = WindowWrapper::new(event_loop, &settings.name, settings.size);
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
                Event::Suspended => surface.suspend(),
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        // Get next frame to render onto
                        if let Some(frame) = surface.acquire_next_frame(&context) {
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
                        } else {
                            warn!("No surface yet, but redraw was requested!");
                        }

                        window.request_redraw();
                    }
                    _ => debug!("Unhandled WindowEvent received: {:#?}", event),
                },
                Event::NewEvents(a) => {
                    debug!("Start cause: {:#?}", a);

                    match app.as_mut() {
                        Some(app) => app.update(),
                        None => warn!("Trying to update non-existing app!"),
                    }
                }
                _ => debug!("Unhandled Event received: {:#?}", event),
            });

        Ok(())
    }
}