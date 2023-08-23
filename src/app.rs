use std::{env::var, sync::Arc, time::Instant};
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{app_window::AppWindow, engine::Engine, AppConfig, APP_NAME};

pub struct App {
    event_loop: EventLoop<()>,
    window: Arc<AppWindow>,
    engine: Arc<Engine>,
    should_run: bool,
    fps: u32,
    delta_time: f64,
    clear_colour: Color,
    clear_colour_index: u32,
    clear_colour_increasing: bool,
}

impl App {
    pub async fn from_app_config_default_path() -> Self {
        #[cfg(target_os = "windows")]
        let mut default_config_path = var("APPDATA")
            .and_then(|x| Ok(format!("{x}/{APP_NAME}")))
            .expect("Failed finding default configuration directory!");

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let mut default_config_path = var("XDG_CONFIG_HOME")
            .or_else(|_| var("HOME").map(|home| format!("{home}/.config/{APP_NAME}")))
            .expect("Failed finding default configuration directory!");

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        compile_error!("::: FIXME ::: OTHER PLATFORMS OTHER THAN WINDOWS, LINUX, MACOS DON'T HAVE A DEFAULT CONFIG PATH CONFIGURED YET! ::: FIXME :::");

        default_config_path = format!("{default_config_path}/app_config.toml");
        log::debug!("Default config path: {default_config_path}");

        let app_config = AppConfig::read_or_write_default(&default_config_path);

        App::from_app_config(app_config).await
    }

    pub async fn from_app_config_path(app_config_path: &str) -> Self {
        let app_config = AppConfig::read_or_write_default(app_config_path);

        Self::from_app_config(app_config).await
    }

    pub async fn from_app_config(app_config: AppConfig) -> Self {
        let event_loop = EventLoop::new();
        let window = Arc::new(AppWindow::build_and_open(
            "WGPU",
            app_config.get_physical_size(),
            false,
            true,
            app_config.convert_fullscreen(&event_loop),
            &event_loop,
        ));

        let engine = Arc::new(Engine::initialize(window.clone()).await);
        engine.configure_surface().await;

        Self {
            event_loop,
            window,
            engine,
            should_run: false,
            fps: 0,
            delta_time: 0.0,
            clear_colour: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            clear_colour_index: 0,
            clear_colour_increasing: true,
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
                return;
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
                    "WGPU - UPS: {}/s (Δ {}s)",
                    self.fps, self.delta_time
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
                                load: LoadOp::Clear(self.clear_colour),
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
                    match self.clear_colour_index {
                        0 => {
                            if self.clear_colour_increasing {
                                self.clear_colour.r += INCREASE_RATE;
                            } else {
                                self.clear_colour.r -= INCREASE_RATE;
                            }

                            if self.clear_colour.r >= 1.0 || self.clear_colour.r <= 0.0 {
                                self.clear_colour_increasing = !self.clear_colour_increasing;
                            }

                            if self.clear_colour.r <= 0.1 && !self.clear_colour_increasing {
                                self.clear_colour_index = 1;
                                self.clear_colour_increasing = true;
                                self.clear_colour.r = 0.0;
                            }
                        }
                        1 => {
                            if self.clear_colour_increasing {
                                self.clear_colour.g += INCREASE_RATE;
                            } else {
                                self.clear_colour.g -= INCREASE_RATE;
                            }

                            if self.clear_colour.g >= 1.0 || self.clear_colour.g <= 0.0 {
                                self.clear_colour_increasing = !self.clear_colour_increasing;
                            }

                            if self.clear_colour.g <= 0.1 && !self.clear_colour_increasing {
                                self.clear_colour_index = 2;
                                self.clear_colour_increasing = true;
                                self.clear_colour.g = 0.0;
                            }
                        }
                        2 => {
                            if self.clear_colour_increasing {
                                self.clear_colour.b += INCREASE_RATE;
                            } else {
                                self.clear_colour.b -= INCREASE_RATE;
                            }

                            if self.clear_colour.b >= 1.0 || self.clear_colour.b <= 0.0 {
                                self.clear_colour_increasing = !self.clear_colour_increasing;
                            }

                            if self.clear_colour.b <= 0.1 && !self.clear_colour_increasing {
                                self.clear_colour_index = 0;
                                self.clear_colour_increasing = true;
                                self.clear_colour.b = 0.0;
                            }
                        }
                        _ => (),
                    }

                    // If redrawing finished -> request to redraw next cycle
                    self.window.get_window().request_redraw();
                }
                _ => (),
            }
        });
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        todo!()
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        todo!()
    }

    pub fn update(&mut self) {
        todo!()
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!()
    }
}
