use std::sync::Arc;

use wgpu::{
    Color, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{DeviceId, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    app::{app_config::AppConfig, app_window::AppWindow},
    engine::Engine,
};

use self::{
    app_context::AppContext,
    app_cycle_counter::AppCycleCounter,
    app_input_handler::{keyboard_input_handler::AppKeyboardInputHandler, AppInputHandler},
    app_world::{clear_screen_object::ClearScreenObject, AppWorld},
};

pub mod app_config;
pub mod app_context;
pub mod app_cycle_counter;
pub mod app_input_handler;
pub mod app_window;
pub mod app_world;

pub struct App {
    app_config: AppConfig,
    app_cycle_counter: AppCycleCounter,
    app_input_handler: AppInputHandler,
    app_world: AppWorld,
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
            app_cycle_counter: AppCycleCounter::default(),
            app_input_handler: AppInputHandler::new(),
            app_world: AppWorld::new(),
        }
    }

    pub async fn spawn_world(&mut self) {
        let clear_screen = ClearScreenObject::new();
        let clear_screen_boxed = Box::new(clear_screen);
        self.app_world.spawn_object(clear_screen_boxed);
    }

    pub async fn hijack_thread_and_run(mut self) {
        // Event Loop
        let event_loop = EventLoop::new();

        // Window creation
        let fullscreen = match &self.app_config.monitor_config {
            Some(x) => Some(x.fullscreen.to_winit_fullscreen(&event_loop, &x)),
            None => None,
        };
        let size = self.app_config.window_config.to_physical_size();
        let window = Arc::new(AppWindow::build_and_open(
            "WGPU",
            size,
            false,
            true,
            fullscreen,
            &event_loop,
        ));

        // Engine creation
        let engine = Arc::new(Engine::initialize(window.clone()).await);
        engine.configure_surface();

        // << Cycle Calculation >>
        self.app_cycle_counter.reset();

        self.spawn_world().await;

        // TODO: FPS Limit cycle calculation
        // I.e. Update every cycle, but render only after delta time is past

        // App Context
        let mut app_context = AppContext::default();
        app_context.clear_colour = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        app_context.clear_colour_index = 0;
        app_context.clear_colour_increasing = true;

        

        event_loop.run(move |event, _target, control_flow| {
            // Immediately start a new cycle once a loop is completed.
            // Ideal for games, but more resource intensive.
            *control_flow = ControlFlow::Poll;

            if let Some((delta_time, ups)) = self.app_cycle_counter.tick() {
                // Update Window Title
                window.get_window().set_title(&format!(
                    "WGPU @ {} - UPS: {}/s (Δ {}s)",
                    engine.get_adapter().get_info().backend.to_str(), ups, delta_time
                ));

                // Update performance outputs
                log::debug!("UPS: {}/s (delta time: {}s)", ups, delta_time);
            }

            // << Events >>
            match event {
                Event::WindowEvent { window_id, event } => {
                    // Validate that the window ID match.
                    // Should only be different if multiple windows are used.
                    if window_id != window.get_window().id() {
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
                        } => {
                            self.handle_keyboard_input(device_id, input, is_synthetic);
                        }
                        _ => (),
                    }
                }
                Event::RedrawRequested(window_id) => {
                    // Validate that the window ID match.
                    // Should only be different if multiple windows are used.
                    if window_id != window.get_window().id() {
                        log::warn!("A window with an ID not matching our window wants to be redrawn by us ... Skipping?");
                        return;
                    }

                    self.handle_redraw(engine.clone());
                }
                Event::RedrawEventsCleared => {
                    // Request to redraw the next cycle
                    window.get_window().request_redraw();
                }
                _ => (),
            }
            
            // Updates!
            if let Some(r_control_flow) = &self.handle_update() {
                if *control_flow != *r_control_flow {
                    log::warn!("Switching control flow from {:?} to {:?}", *control_flow, r_control_flow);
                    *control_flow = *r_control_flow;
                }
            }

            window.get_window().request_redraw();
        });
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
        log::debug!(
            "Resize detected! Changing from {}x{} to {}x{}",
            self.app_config.window_config.size.0,
            self.app_config.window_config.size.1,
            &new_size.width,
            &new_size.height
        );

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

    pub fn get_app_config(&self) -> &AppConfig {
        &self.app_config
    }

    fn handle_update(&mut self) -> Option<ControlFlow> {
        // Exit condition
        if self
            .app_input_handler
            .are_all_keys_pressed(&vec![VirtualKeyCode::LAlt, VirtualKeyCode::F4])
            || self
                .app_input_handler
                .is_key_pressed(&VirtualKeyCode::Escape)
        {
            log::warn!("Exit condition reached!");
            return Some(ControlFlow::Exit);
        }

        // World Spawn
        if self
            .app_input_handler
            .is_key_pressed(&VirtualKeyCode::Space)
        {
            let object = ClearScreenObject::new();
            let object_boxed = Box::new(object);
            self.app_world.spawn_object(object_boxed);

            log::debug!("Object: {}", self.app_world.count_object());
            log::debug!("Updateable: {}", self.app_world.count_updateable());
            log::debug!("Renderable: {}", self.app_world.count_renderable());
        }

        self.app_world.call_updateables();

        None
    }

    fn handle_redraw(&mut self, engine: Arc<Engine>) {
        let output_surface_texture = engine
            .get_surface()
            .get_current_texture()
            .expect("failed acquiring current texture of target window");

        let output_surface_texture_view = output_surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        // Build command buffers for frame
        let command_buffers = self.app_world.call_renderables(engine.clone(), &output_surface_texture_view);

        // Submit command buffer
        engine.get_queue().submit(command_buffers);
        output_surface_texture.present();
    }
}
