use std::iter::once;

use cgmath::Deg;
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
    Camera, EngineError, EngineResult, TAmbientLight, TComputingEngine, TPointLight,
    TRenderingEngine, TTexture, TextureHelper, WGPURenderingEngine, Projection,
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
    camera: Camera,
    #[cfg(debug_assertions)]
    last_draw_calls: u32,
    #[cfg(debug_assertions)]
    last_triangle_count: u32,
}

impl App {
    pub fn run<S>(name: S, world_builder: WorldBuilder) -> EngineResult<()>
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

        let world = world_builder.build(rendering_engine.logical_device());

        let timer = Timer::new();

        let input_handler = InputHandler::new();

        let projection = Projection::new(
            window.inner_size().width,
            window.inner_size().height,
            Deg(45.0),
            0.1,
            100.0,
        );
        let camera = Camera::new(rendering_engine.logical_device(), (0.0, 2.0, 10.0), Deg(-90.0), Deg(-20.0), 0.1, 0.05, projection);

        let mut app = Self {
            name,
            world,
            rendering_engine,
            timer,
            input_handler,
            camera,
            #[cfg(debug_assertions)]
            last_draw_calls: 0,
            #[cfg(debug_assertions)]
            last_triangle_count: 0,
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
                        .keyboard_input_handler_mut()
                        .handle_keyboard_input(input),
                        WindowEvent::CursorMoved { position,  .. } => app.input_handler.mouse_input_handler_mut().handle_cursor_moved(position),
                        WindowEvent::CursorEntered { .. } =>app.input_handler.mouse_input_handler_mut().handle_cursor_entered(),
                        WindowEvent::CursorLeft { .. } => app.input_handler.mouse_input_handler_mut().handle_cursor_left(),
                        WindowEvent::MouseWheel { delta, phase, .. } => app.input_handler.mouse_input_handler_mut().handle_mouse_scroll(phase, delta)   ,
                        WindowEvent::MouseInput { state, button, .. } => 
                            app.input_handler.mouse_input_handler_mut().handle_mouse_input(state, button),
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
        WindowBuilder::new()
            .with_active(active)
            .with_visible(visible)
            .with_title(title)
            .with_inner_size(size)
            .with_maximized(maximized)
            .with_resizable(resizable)
            .build(event_loop)
            .map_err(EngineError::WinitOSError)
    }

    fn handle_resize(&mut self, new_size: &PhysicalSize<u32>, window: &Window) {
        log::info!(
            "Resize detected! Changing from {:?} to {:?} (if valid)!",
            window.inner_size(),
            new_size
        );

        if new_size.width == 0 || new_size.height == 0 {
            log::error!("Invalid new window size received!");
            return;
        }

        if !self.rendering_engine.device().poll(MaintainBase::Wait) {
            log::error!("Failed to poll device before resizing!");
            return;
        }

        // Apply config changes and reconfigure surface
        let mut current_config = self.rendering_engine.surface_configuration().clone();
        current_config.width = new_size.width;
        current_config.height = new_size.height;
        self.rendering_engine
            .set_surface_configuration(current_config);
        self.rendering_engine.reconfigure_surface();

        // Change projection
        let old_projection = self.camera.projection();
        let projection = Projection::new(
            new_size.width, 
            new_size.height,
            old_projection.fovy(),
            old_projection.znear(),
            old_projection.zfar(),
        );
        self.camera.set_projection(projection);
    }

    fn handle_redraw(&mut self) -> EngineResult<()> {
        #[cfg(debug_assertions)]
        {
            self.last_draw_calls = 0;
            self.last_triangle_count = 0;
        }

        let surface_texture = self.rendering_engine.surface_texture()?;
        let surface_texture_view = surface_texture.make_texture_view();

        let mut command_encoder =
            self.rendering_engine
                .device()
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
                        load: LoadOp::Clear(self.world.clear_color()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: self
                        .rendering_engine
                        .depth_texture()
                        .expect("Depth Texture gone missing!")
                        .view(),
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(self.rendering_engine.render_pipeline());

            // Call entity renderables
            let (meshes, ambient_light, point_lights) = self
                .world
                .prepare_render_and_collect_data(self.rendering_engine.logical_device());

            meshes.iter().for_each(|x| {
                #[cfg(debug_assertions)]
                {
                    self.last_draw_calls += 1;
                    self.last_triangle_count += x.instance_count() / 3;
                }

                // Vertex & Instance Buffer
                render_pass.set_vertex_buffer(0, x.vertex_buffer().slice(..));
                render_pass.set_vertex_buffer(1, x.instance_buffer().slice(..));

                // Index Buffer
                render_pass.set_index_buffer(x.index_buffer().slice(..), IndexFormat::Uint32);

                // Texture / Material
                render_pass.set_bind_group(0, x.material().bind_group(), &[]);

                // Camera
                render_pass.set_bind_group(1, self.camera.bind_group(), &[]);

                // Ambient Light
                render_pass.set_bind_group(2, ambient_light.bind_group(), &[]);

                // Point Light
                render_pass.set_bind_group(3, point_lights[0].bind_group(), &[]);

                render_pass.draw_indexed(0..x.index_count(), 0, 0..x.instance_count());
            });
        }

        let command_buffer = command_encoder.finish();
        self.rendering_engine.queue().submit(once(command_buffer));
        surface_texture.present();

        Ok(())
    }

    fn handle_main_events_cleared(&mut self, window: &Window) {
        // Fast (i.e. by-cycle) updates
        self.world.call_updateable(
            UpdateFrequency::Fast,
            self.timer.current_delta_time(),
            &self.input_handler,
            &mut self.camera,
            self.rendering_engine.logical_device(),
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
                    self.rendering_engine.adapter().get_info().backend.to_str(),
                    ups,
                    delta_time
                ));

                #[cfg(debug_assertions)]
                {
                    log::debug!("Draw Calls: {}", self.last_draw_calls());
                    log::debug!("Triangle Count: {}", self.last_triangle_count());
                }
            }

            // Slow (i.e. by-second) updates
            self.world.call_updateable(
                UpdateFrequency::Slow,
                delta_time,
                &self.input_handler,
                &mut self.camera,
                self.rendering_engine.logical_device(),
            );
        }
    }

    #[cfg(debug_assertions)]
    pub fn last_draw_calls(&self) -> u32 {
        self.last_draw_calls
    }

    #[cfg(debug_assertions)]
    pub fn last_triangle_count(&self) -> u32 {
        self.last_triangle_count
    }
}
