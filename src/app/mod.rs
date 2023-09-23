use std::time::Instant;

use wgpu::MaintainBase;
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::engine::{
    EngineError, EngineResult, TComputingEngine, TRenderingEngine, WGPURenderingEngine,
};

mod app_object;
pub use app_object::*;

mod app_world;
pub use app_world::*;

pub struct App {
    name: String,
    world: AppWorld,
    rendering_engine: WGPURenderingEngine,
    last_time: Instant,
    current_cycle_count: u64,
    current_delta_time: f64,
}

impl App {
    pub fn run<S>(name: S, world: AppWorld) -> EngineResult<()>
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

        let mut app = Self {
            name,
            world,
            rendering_engine,
            last_time: Instant::now(),
            current_cycle_count: 0,
            current_delta_time: 0.0,
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
                    WindowEvent::KeyboardInput { input, .. } => app.handle_keyboard_input(input),
                    _ => (),
                },
                Event::RedrawRequested(..) => app.handle_redraw(),
                Event::RedrawEventsCleared => app.handle_redraw_events_cleared(),
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

    pub fn get_name(&self) -> &String {
        &self.name
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

    fn handle_keyboard_input(&mut self, input: KeyboardInput) {
        // TODO
    }

    fn handle_redraw(&mut self) {
        // TODO
    }

    fn handle_redraw_events_cleared(&mut self) {
        // TODO
    }

    fn handle_main_events_cleared(&mut self, window: &Window) {
        let elapsed = self.last_time.elapsed();
        self.last_time = Instant::now();

        self.current_delta_time += elapsed.as_secs_f64();
        self.current_cycle_count += 1;

        // TODO: Fast/Dynamic updates

        if self.current_delta_time >= 1.0 {
            #[cfg(debug_assertions)]
            {
                // Update performance outputs
                log::debug!(
                    "UPS: {}/s (delta time: {}s)",
                    self.current_cycle_count,
                    self.current_delta_time
                );

                // Update Window Title
                window.set_title(&format!(
                    "WGPU @ {} - UPS: {}/s (Δ {}s)",
                    self.rendering_engine
                        .get_adapter()
                        .get_info()
                        .backend
                        .to_str(),
                    self.current_cycle_count,
                    self.current_delta_time
                ));
            }

            // TODO: Slow updates

            self.current_cycle_count = 0;
            self.current_delta_time -= 1.0;
        }
    }
}
