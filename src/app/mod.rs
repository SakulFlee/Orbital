use std::{sync::Arc, time::Instant};
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    app::{app_config::AppConfig, app_window::AppWindow},
    engine::Engine,
};

use self::app_context::AppContext;

pub mod app_config;
pub mod app_context;
pub mod app_window;

pub struct App {
    app_config: AppConfig,
    event_loop: EventLoop<()>,
    window: Arc<AppWindow>,
    engine: Arc<Engine>,
    should_run: bool,
    fps: u32,
    delta_time: f64,
    app_context: AppContext,
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
        let event_loop = EventLoop::new();

        let fullscreen = match &app_config.monitor_config {
            Some(x) => Some(x.fullscreen.to_winit_fullscreen(&event_loop, &x)),
            None => None,
        };

        let size = app_config.window_config.to_physical_size();
        let window = Arc::new(AppWindow::build_and_open(
            "WGPU",
            size,
            false,
            true,
            fullscreen,
            &event_loop,
        ));

        let engine = Arc::new(Engine::initialize(window.clone()).await);
        engine.configure_surface();

        let mut app_context = AppContext::default();
        app_context.clear_colour = Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        app_context.clear_colour_index = 0;
        app_context.clear_colour_increasing = true;

        Self {
            app_config,
            event_loop,
            window,
            engine,
            should_run: false,
            fps: 0,
            delta_time: 0.0,
            app_context,
        }
    }

    pub fn start(mut self) {
        self.should_run = true;

        // << FPS Calculation >>
        // Last "now time"
        let mut last_cycle_time = Instant::now();
        // Iteration count per cycle
        let mut cycle_count: u32 = 0;

        self.event_loop.run(move |event, _target, control_flow| {
            // << Control Flow >>
            if self.should_run {
                // Immediately start a new cycle once a loop is completed.
                // Ideal for games, but more resource intensive.
                *control_flow = ControlFlow::Poll;
            } else {
                // Exit is requested.
                *control_flow = ControlFlow::ExitWithCode(0);
            }

            // << Cycle Calculation >>
            // Increase delta count and take "now time"
            cycle_count += 1;
            let now_cycle_time = Instant::now();
            // Calculate duration since last cycle time
            let delta_duration = now_cycle_time.duration_since(last_cycle_time);
            // Add difference to delta time
            self.delta_time = self.delta_time + delta_duration.as_secs_f64();

            // If delta time is over a second, end the cycle
            if self.delta_time >= 1.0 {
                // Update FPS counter
                self.fps = cycle_count;

                // Update Window Title
                self.window.get_window().set_title(&format!(
                    "WGPU @ {} - UPS: {}/s (Δ {}s)",
                    self.engine.get_adapter().get_info().backend.to_str(), self.fps, self.delta_time
                ));

                // Update performance outputs
                log::debug!("UPS: {}/s (delta time: {}s)", self.fps, self.delta_time);

                // One second has past, subtract that
                self.delta_time -= 1.0;
                // Reset cycle
                cycle_count = 0;
            }
            // Update cycle time with now time
            last_cycle_time = now_cycle_time;

            // << Variables >>
            let mut resize_to: Option<PhysicalSize<u32>> = None;

            // << Events >>
            match event {
                Event::WindowEvent { window_id, event } => {
                    
                    // log::debug!("Window Event :: Window ID: {window_id:?}, Event: {event:?}");

                    // Validate that the window ID match.
                    // Should only be different if multiple windows are used.
                    if window_id != self.window.get_window().id() {
                        log::warn!("Invalid window ID for 'Window Event :: Window ID: {window_id:?}, Event: {event:?}'");
                        return;
                    }

                    match event {
                        WindowEvent::CloseRequested => self.should_run = false,
                        WindowEvent::Resized(new_size) => {
                            resize_to = Some(new_size);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            resize_to = Some(*new_inner_size);
                        }
                        WindowEvent::KeyboardInput {  
                            input: KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => self.should_run = false,
                        _ => (),
                    }
                }
                Event::RedrawRequested(window_id) => {
                    log::debug!("Redraw Requested :: Window ID: {window_id:?}");

                    // Validate that the window ID match.
                    // Should only be different if multiple windows are used.
                    if window_id != self.window.get_window().id() {
                        log::warn!("Invalid window ID for above's Event!");
                        return;
                    }

                    // TODO: Rendering goes here
                    let output_surface_texture = self
                        .engine
                        .get_surface()
                        .get_current_texture()
                        .expect("failed acquiring current texture of target window");

                    let output_surface_texture_view = output_surface_texture
                        .texture
                        .create_view(&TextureViewDescriptor::default());

                    let mut command_encoder = self.engine.get_device().create_command_encoder(
                        &CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        },
                    );

                    command_encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &output_surface_texture_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(self.app_context.clear_colour),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                    let command_buffer = command_encoder.finish();

                    // Submit command buffer
                    self.engine.get_queue().submit(vec![command_buffer]);
                    output_surface_texture.present();
                }
                Event::RedrawEventsCleared => {
                    // Update colour variable
                    const INCREASE_RATE: f64 = 0.005;
                    match self.app_context.clear_colour_index {
                        0 => {
                            if self.app_context.clear_colour_increasing {
                                self.app_context.clear_colour.r += INCREASE_RATE;
                            } else {
                                self.app_context.clear_colour.r -= INCREASE_RATE;
                            }

                            if self.app_context.clear_colour.r >= 1.0 || self.app_context.clear_colour.r <= 0.0 {
                                self.app_context.clear_colour_increasing = !self.app_context.clear_colour_increasing;
                            }

                            if self.app_context.clear_colour.r <= 0.1 && !self.app_context.clear_colour_increasing {
                                self.app_context.clear_colour_index = 1;
                                self.app_context.clear_colour_increasing = true;
                                self.app_context.clear_colour.r = 0.0;
                            }
                        }
                        1 => {
                            if self.app_context.clear_colour_increasing {
                                self.app_context.clear_colour.g += INCREASE_RATE;
                            } else {
                                self.app_context.clear_colour.g -= INCREASE_RATE;
                            }

                            if self.app_context.clear_colour.g >= 1.0 || self.app_context.clear_colour.g <= 0.0 {
                                self.app_context.clear_colour_increasing = !self.app_context.clear_colour_increasing;
                            }

                            if self.app_context.clear_colour.g <= 0.1 && !self.app_context.clear_colour_increasing {
                                self.app_context.clear_colour_index = 2;
                                self.app_context.clear_colour_increasing = true;
                                self.app_context.clear_colour.g = 0.0;
                            }
                        }
                        2 => {
                            if self.app_context.clear_colour_increasing {
                                self.app_context.clear_colour.b += INCREASE_RATE;
                            } else {
                                self.app_context.clear_colour.b -= INCREASE_RATE;
                            }

                            if self.app_context.clear_colour.b >= 1.0 || self.app_context.clear_colour.b <= 0.0 {
                                self.app_context.clear_colour_increasing = !self.app_context.clear_colour_increasing;
                            }

                            if self.app_context.clear_colour.b <= 0.1 && !self.app_context.clear_colour_increasing {
                                self.app_context.clear_colour_index = 0;
                                self.app_context.clear_colour_increasing = true;
                                self.app_context.clear_colour.b = 0.0;
                            }
                        }
                        _ => (),
                    }

                    if resize_to.is_some() {
                        let new_size = resize_to.unwrap();
                        resize_to = None;

                        log::debug!("Resize detected! Changing from {}x{} to {}x{}", self.app_config.window_config.size.0, self.app_config.window_config.size.1, &new_size.width, &new_size.height);

                            // Update config
                            self.app_config.window_config.size = new_size.into();
                            if self.app_config.monitor_config.is_some() {
                                self.app_config.monitor_config.as_mut().unwrap().size = new_size.into();
                            }
                            self.app_config.write_to_path(&AppConfig::request_default_path());

                            // Skip redrawing and reconfigure the surface
                            self.engine.configure_surface();
                    } else {
                        // Request to redraw the next cycle
                        self.window.get_window().request_redraw();
                    }
                }
                _ => (),
            }
        });

        // Return self since this function takes ownership.
        // This is required to access data **after** execution.
        //
        // Note: Rust thinks this line is unreachable, but it actually is.
        // #[allow(unreachable_code)]
        // return self;
    }

    pub fn get_app_config(&self) -> &AppConfig {
        &self.app_config
    }
}
