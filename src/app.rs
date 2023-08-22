use std::{env::var, time::Instant};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
};

use crate::{app_window::AppWindow, AppConfig, APP_NAME};

pub struct App {
    event_loop: EventLoop<()>,
    window: AppWindow,
    should_run: bool,
    fps: u32,
    delta_time: f64,
}

impl App {
    pub fn from_app_config_default_path() -> Self {
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

        App::from_app_config(app_config)
    }

    pub fn from_app_config_path(app_config_path: &str) -> Self {
        let app_config = AppConfig::read_or_write_default(app_config_path);

        Self::from_app_config(app_config)
    }

    pub fn from_app_config(app_config: AppConfig) -> Self {
        let event_loop = EventLoop::new();
        let window = AppWindow::build_and_open(
            "WGPU",
            app_config.get_physical_size(),
            false,
            true,
            app_config.convert_fullscreen(&event_loop),
            &event_loop,
        );

        Self {
            event_loop,
            window,
            should_run: false,
            fps: 0,
            delta_time: 0.0,
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
            // Immediately start a new cycle once a loop is completed.
            // Ideal for games, but more resource intensive.
            *control_flow = ControlFlow::Poll;

            // <<< Cycle Calculation >>>
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
