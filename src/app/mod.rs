use std::iter::once;

use wgpu::{
    CommandEncoderDescriptor, IndexFormat, LoadOp, MaintainBase, Operations,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
};
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::engine::{
    EngineError, EngineResult, TComputingEngine, TRenderingEngine, TTexture, TextureHelper,
    WGPURenderingEngine,
};

mod input;
pub use input::*;

mod timer;
pub use timer::*;

mod world;
pub use world::*;

pub struct App {
    name: String,
    world: World,
    rendering_engine: WGPURenderingEngine,
    timer: Timer,
    input_handler: InputHandler,
}

impl App {
    pub fn run<S>(name: S, world: World) -> EngineResult<()>
    where
        S: Into<String>,
    {
        let name: String = name.into();

        let event_loop = Self::make_event_loop();
        let window = Self::make_window(
            &event_loop,
            true,
            true,
            &name,
            PhysicalSize::new(1280, 720),
            false,
            false,
        )?;

        let rendering_engine = WGPURenderingEngine::new(&window)?;

        let timer = Timer::new();

        let input_handler = InputHandler::new();

        let mut app = Self {
            name,
            world,
            rendering_engine,
            timer,
            input_handler,
        };

        event_loop.run(move |event, _, control_flow| {
            // Immediately start a new cycle once a loop is completed.
            // Ideal for games, but more resource intensive.
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::ExitWithCode(0),
                    WindowEvent::Resized(new_size) => app.handle_resize(&new_size, &window),
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size: new_size,
                        ..
                    } => app.handle_resize(new_size, &window),
                    WindowEvent::KeyboardInput { input, .. } => app
                        .input_handler
                        .get_keyboard_input_handler()
                        .handle_keyboard_input(input),
                    _ => (),
                },
                Event::RedrawRequested(..) => {
                    if let Err(e) = app.handle_redraw() {
                        log::error!("An error has occurred while rendering!\nThe error encountered was:\n{:?}", e);
                        log::warn!("Engine potentially unstable. Restart app if further issues occur!");
                    }
                },
                Event::RedrawEventsCleared => window.request_redraw(),
                Event::MainEventsCleared => app.handle_main_events_cleared(&window),
                _ => (),
            }
        });
    }

    fn make_event_loop() -> EventLoop<()> {
        EventLoop::new()
    }

    fn make_window<T, S>(
        event_loop: &EventLoop<()>,
        active: bool,
        visible: bool,
        title: T,
        size: S,
        maximized: bool,
        resizable: bool,
    ) -> EngineResult<Window>
    where
        T: Into<String>,
        S: Into<Size>,
    {
        Ok(WindowBuilder::new()
            .with_active(active)
            .with_visible(visible)
            .with_title(title)
            .with_inner_size(size)
            .with_maximized(maximized)
            .with_resizable(resizable)
            .build(&event_loop)
            .map_err(|e| EngineError::WinitOSError(e))?)
    }

    fn handle_resize(&mut self, new_size: &PhysicalSize<u32>, window: &Window) {
        log::info!(
            "Resize detected! Changing from {:?} to {:?} (if valid)!",
            window.inner_size(),
            new_size
        );

        if new_size.width <= 0 || new_size.height <= 0 {
            log::error!("Invalid new window size received!");
            return;
        }

        if !self.rendering_engine.get_device().poll(MaintainBase::Wait) {
            log::error!("Failed to poll device before resizing!");
            return;
        }

        // Apply config changes and reconfigure surface
        let mut current_config = self.rendering_engine.get_surface_configuration().clone();
        current_config.width = new_size.width;
        current_config.height = new_size.height;
        self.rendering_engine
            .set_surface_configuration(current_config);
        self.rendering_engine.reconfigure_surface();
    }

    fn handle_redraw(&mut self) -> EngineResult<()> {
        let surface_texture = self.rendering_engine.get_surface_texture()?;
        let surface_texture_view = surface_texture.make_texture_view();

        let mut command_encoder =
            self.rendering_engine
                .get_device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.world.get_clear_color()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self
                        .rendering_engine
                        .get_depth_texture()
                        .expect("Depth Texture gone missing!")
                        .get_view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(self.rendering_engine.get_render_pipeline());

            render_pass.set_bind_group(
                1,
                &self.rendering_engine.get_camera().get_bind_group(),
                &[],
            );

            // TODO: Uniform buffers like Depth Buffer, Camera, etc. (AS NEEDED)

            // Call entity renderables
            let meshes = self.world.prepare_render_and_collect_meshes(
                self.rendering_engine.get_device(),
                self.rendering_engine.get_queue(),
            );

            meshes.iter().for_each(|x| {
                // Vertex & Instance Buffer
                render_pass.set_vertex_buffer(0, x.get_vertex_buffer().slice(..));
                render_pass.set_vertex_buffer(1, x.get_instance_buffer().slice(..));

                // Index Buffer
                render_pass.set_index_buffer(x.get_index_buffer().slice(..), IndexFormat::Uint32);

                // Texture / Material
                render_pass.set_bind_group(0, &x.get_material().get_bind_group(), &[]);

                render_pass.draw_indexed(0..x.get_index_count(), 0, 0..x.get_instance_count());
            });
        }

        let command_buffer = command_encoder.finish();
        self.rendering_engine
            .get_queue()
            .submit(once(command_buffer));
        surface_texture.present();

        Ok(())
    }

    fn handle_main_events_cleared(&mut self, window: &Window) {
        // Fast (i.e. by-cycle) updates
        self.world.call_updateable(
            UpdateFrequency::Fast,
            self.timer.get_current_delta_time(),
            &self.input_handler,
        );

        if let Some((delta_time, ups)) = self.timer.tick() {
            #[cfg(debug_assertions)]
            {
                // Update performance outputs
                log::debug!("UPS: {}/s (delta time: {}s)", ups, delta_time);

                // Update Window Title
                window.set_title(&format!(
                    "{} @ {} - UPS: {}/s (Δ {}s)",
                    self.name,
                    self.rendering_engine
                        .get_adapter()
                        .get_info()
                        .backend
                        .to_str(),
                    ups,
                    delta_time
                ));
            }

            // Slow (i.e. by-second) updates
            self.world
                .call_updateable(UpdateFrequency::Slow, delta_time, &self.input_handler);
        }
    }
}
