use std::{sync::Arc, time::Instant};

use wgpu::TextureViewDescriptor;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceId, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowId},
};

use crate::{app::app_config::AppConfig, engine::Engine, APP_NAME};

use self::{
    app_input_handler::{keyboard_input_handler::AppKeyboardInputHandler, AppInputHandler},
    app_world::{clear_screen_object::ClearScreenObject, AppWorld},
};

pub mod app_config;
pub mod app_context;
pub mod app_input_handler;
pub mod app_world;

pub struct App {
    app_config: AppConfig,
    app_input_handler: AppInputHandler,
    app_world: AppWorld,
    app_window: Option<Arc<Window>>,

    last_time: Instant,
    delta_time: f64,
    ups: u64,
}

impl App {
    pub async fn from_app_config_default_path() -> Self {
        let default_config_path = AppConfig::request_default_path();
        let app_config = AppConfig::read_or_write_default(&default_config_path);

        App::from_app_config(app_config).await
    }

    pub async fn from_app_config_path(app_config_path: &str) -> Self {
        let app_config = AppConfig::read_or_write_default(app_config_path);

        Self::from_app_config(app_config).await
    }

    pub async fn from_app_config(app_config: AppConfig) -> Self {
        Self {
            app_config,
            app_input_handler: AppInputHandler::new(),
            app_world: AppWorld::new(),
            app_window: None,
            last_time: Instant::now(),
            delta_time: 0.0,
            ups: 0,
        }
    }

    pub async fn spawn_world(&mut self) {
        let clear_screen = ClearScreenObject::new();
        let clear_screen_boxed = Box::new(clear_screen);
        self.app_world.spawn_object(clear_screen_boxed);
    }

    pub async fn hijack_thread_and_run(mut self) {
        // Event Loop & Window creation
        let event_loop = EventLoop::new();
        let window_arc = Arc::new(self.build_window(&event_loop));
        self.app_window = Some(window_arc.clone());

        // Engine creation
        let engine = Arc::new(Engine::initialize(window_arc.clone()).await);
        engine.configure_surface();

        let engine_backend = engine.get_adapter().get_info().backend.to_str();
        log::info!("Engine Backend: {engine_backend}");

        // World
        self.spawn_world().await;

        // << Cycle Calculation >>
        self.last_time = Instant::now();
        self.delta_time = 0.0;
        self.ups = 0;

        event_loop.run(move |event, _target, control_flow| {
            // Immediately start a new cycle once a loop is completed.
            // Ideal for games, but more resource intensive.
            *control_flow = ControlFlow::Poll;

            // << Events >>
            match event {
                Event::WindowEvent { window_id, event } => {
                    // Validate that the window ID match.
                    // Should only be different if multiple windows are used.
                    if window_id != window_arc.clone().id() {
                        log::warn!("Invalid window ID for 'Window Event :: Window ID: {window_id:?}, Event: {event:?}'");
                        return;
                    }

                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::ExitWithCode(0),
                        WindowEvent::Resized(new_size) => self.handle_resize(new_size, &engine),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => self.handle_resize(*new_inner_size, &engine),
                        WindowEvent::KeyboardInput {
                            device_id,
                            input,
                            is_synthetic
                        } => self.handle_keyboard_input(device_id, input, is_synthetic),
                        _ => (),
                    }
                }
                Event::RedrawRequested(window_id) => self.handle_redraw(engine.clone(), window_id),
                Event::RedrawEventsCleared => self.handle_redraw_events_cleared(),
                Event::MainEventsCleared => self.handle_main_events_cleared(&engine_backend, control_flow),
                _ => (),
            }
        });
    }

    fn handle_redraw_events_cleared(&mut self) {
        // Request to redraw the next cycle
        self.app_window.clone().unwrap().request_redraw();
    }

    /// Handles the main events cleared event.
    /// This event is triggered after **all** other events have been processed.
    /// This event **should** be relatively consistent in comparison.
    ///
    /// This function is used to update our app, including:
    /// - Updateables
    /// - UPS & Delta calculation
    /// - Performance outputs
    fn handle_main_events_cleared(&mut self, backend_name: &str, control_flow: &mut ControlFlow) {
        // Take now time
        let now = Instant::now();
        // Get the duration of elapsed time since last update
        let elapsed = self.last_time.elapsed();
        // Update last time with now time
        self.last_time = now;

        // Add the elapsed time to the delta time
        self.delta_time += elapsed.as_secs_f64();
        // Increment UPS cycle count
        self.ups += 1;

        // Call updateables
        self.app_world.call_updateables_on_cycle(self.delta_time);

        // If a second has past, call updates
        if self.delta_time >= 1.0 {
            #[cfg(debug_assertions)]
            {
                // Update performance outputs
                log::debug!("UPS: {}/s (delta time: {}s)", self.ups, self.delta_time);

                // Update Window Title
                self.app_window.clone().unwrap().set_title(&format!(
                    "WGPU @ {} - UPS: {}/s (Δ {}s)",
                    backend_name, self.ups, self.delta_time
                ));
            }

            // Check for main keyboard inputs
            if self
                .app_input_handler
                .are_all_keys_pressed(&vec![VirtualKeyCode::LAlt, VirtualKeyCode::F4])
                || self
                    .app_input_handler
                    .is_key_pressed(&VirtualKeyCode::Escape)
            {
                log::warn!("Exit condition reached!");
                *control_flow = ControlFlow::Exit;
            }

            // Call updateables
            self.app_world.call_updateables_on_second(self.delta_time);

            // Reset counters
            self.ups = 0;
            self.delta_time -= 1.0;
        }
    }

    fn handle_keyboard_input(
        &mut self,
        _device_id: DeviceId,
        input: KeyboardInput,
        _is_synthetic: bool,
    ) {
        let input_handler: &mut AppInputHandler = &mut self.app_input_handler;
        let keyboard_handler: &mut AppKeyboardInputHandler =
            input_handler.get_keyboard_input_handler();

        keyboard_handler.handle_keyboard_input(input);
    }

    fn handle_resize(&mut self, new_size: PhysicalSize<u32>, engine: &Engine) {
        log::info!(
            "Resize detected! Changing from {}x{} to {}x{}",
            self.app_config.window_config.size.0,
            self.app_config.window_config.size.1,
            &new_size.width,
            &new_size.height
        );

        if new_size.width <= 0 || new_size.height <= 0 {
            log::error!("Invalid new window size received!");
            return;
        }

        if !engine.get_device().poll(wgpu::MaintainBase::Wait) {
            log::error!("Failed to poll device before resizing!");
            return;
        }

        // Update config
        self.app_config.window_config.size = new_size.into();
        if self.app_config.monitor_config.is_some() {
            self.app_config.monitor_config.as_mut().unwrap().size = new_size.into();
        }
        self.app_config
            .write_to_path(&AppConfig::request_default_path());

        // Reconfigure the surface
        engine.configure_surface();
    }

    fn handle_redraw(&mut self, engine: Arc<Engine>, window_id: WindowId) {
        // Validate that the window ID match.
        // Should only be different if multiple windows are used.
        if window_id != self.app_window.clone().unwrap().id() {
            log::warn!("A window with an ID not matching our window wants to be redrawn by us ... Skipping?");
            return;
        }

        let output_surface_texture = engine
            .get_surface()
            .get_current_texture()
            .expect("failed acquiring current texture of target window");

        let output_surface_texture_view = output_surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Build command buffers for frame
        let command_buffers = self
            .app_world
            .call_renderables(engine.clone(), &output_surface_texture_view);

        // Submit command buffer
        engine.get_queue().submit(command_buffers);
        output_surface_texture.present();
    }

    fn build_window(&self, event_loop: &EventLoop<()>) -> Window {
        let fullscreen = match &self.app_config.monitor_config {
            Some(x) => Some(x.fullscreen.to_winit_fullscreen(&event_loop, &x)),
            None => None,
        };

        let size = self.app_config.window_config.to_physical_size();

        let mut builder = WindowBuilder::new();
        builder = builder.with_active(true);
        builder = builder.with_visible(true);
        builder = builder.with_title(APP_NAME);
        builder = builder.with_inner_size(size);
        builder = builder.with_maximized(false);
        builder = builder.with_resizable(true);

        if fullscreen.is_some() {
            builder = builder.with_fullscreen(fullscreen);
        }

        match builder.build(&event_loop) {
            Ok(window) => window,
            Err(err) => panic!("Window building failed! {:#?}", err),
        }
    }

    pub fn get_app_config(&self) -> &AppConfig {
        &self.app_config
    }
}
