use std::env::var;
use winit::{dpi::PhysicalSize, event::WindowEvent, event_loop::EventLoop};

use crate::{app_window::AppWindow, AppConfig, APP_NAME};

pub struct App {
    event_loop: EventLoop<()>,
    window: AppWindow,
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

        Self { event_loop, window }
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
