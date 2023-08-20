use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
    window::Fullscreen,
};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub width: u32,
    pub height: u32,
    pub fullscreen: Option<FullscreenWrapper>,
    pub last_monitor_x: i32,
    pub last_monitor_y: i32,
    pub last_monitor_refresh_rate: u32,
}

#[derive(Serialize, Deserialize)]
pub enum FullscreenWrapper {
    Borderless,
    Exclusive,
}

impl AppConfig {
    pub fn read_or_write_default(config_path: &str) -> Self {
        match Self::read_from_path(config_path) {
            Some(config) => config,
            None => {
                log::info!("AppConfig not found (or invalid) -> Generating default config!");
                let default_config = AppConfig::default();
                match default_config.write_to_path(config_path) {
                    Some(_) => (),
                    None => {
                        log::warn!(
                            "Failed writing AppConfig to disk; returning default config anyways!"
                        );
                    }
                }
                default_config
            }
        }
    }

    pub fn read_from_str(config_str: &str) -> Option<Self> {
        match toml::from_str(config_str) {
            Ok(x) => Some(x),
            Err(_) => {
                log::error!("Invalid AppConfig!");
                None
            }
        }
    }

    pub fn read_from_path(config_path: &str) -> Option<Self> {
        let file_content = match fs::read_to_string(config_path) {
            Ok(content) => content,
            Err(e) => {
                log::error!("Failed reading AppConfig at {config_path}! ({e})");
                return None;
            }
        };
        Self::read_from_str(&file_content)
    }

    pub fn write_to_str(&self) -> Option<String> {
        match toml::to_string(&self) {
            Ok(content) => Some(content),
            Err(e) => {
                log::error!("Failed parsing AppConfig to String! ({e})");
                None
            }
        }
    }

    pub fn write_to_path(&self, path: &str) -> Option<()> {
        let path = Path::new(path);
        let parent = path.parent().expect("config folder has no parent");

        if !parent.exists() {
            fs::create_dir_all(parent).expect("failed creating parent folder");
        }

        match self.write_to_str() {
            Some(content_str) => match fs::write(path, content_str) {
                Ok(()) => Some(()),
                Err(e) => {
                    log::error!("Failed writing AppConfig to disk! ({e})");
                    None
                }
            },
            None => None,
        }
    }

    pub fn get_physical_size(&self) -> PhysicalSize<u32> {
        PhysicalSize {
            width: self.width,
            height: self.height,
        }
    }

    pub fn get_last_monitor_physical_position(&self) -> PhysicalPosition<i32> {
        PhysicalPosition {
            x: self.last_monitor_x,
            y: self.last_monitor_y,
        }
    }

    pub fn convert_fullscreen(&self, event_loop: &EventLoop<()>) -> Option<Fullscreen> {
        let last_monitor = self.get_last_monitor_physical_position();
        let monitor_handle = event_loop
            .available_monitors()
            .into_iter()
            .find(|x| x.position() == last_monitor)
            .unwrap_or_else(|| {
                event_loop
                    .primary_monitor()
                    .expect("no primary monitor found!")
            });

        let matching_video_mode = monitor_handle
            .video_modes()
            .find(|x| {
                x.refresh_rate_millihertz() == self.last_monitor_refresh_rate
                    && x.size() == self.get_physical_size()
            })
            .unwrap_or(
                monitor_handle
                    .video_modes()
                    .next()
                    .expect("no video modes found"),
            );

        match &self.fullscreen {
            Some(fullscreen_wrapper) => match fullscreen_wrapper {
                FullscreenWrapper::Borderless => Some(Fullscreen::Borderless(Some(monitor_handle))),
                FullscreenWrapper::Exclusive => Some(Fullscreen::Exclusive(matching_video_mode)),
            },
            None => None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 1080,
            height: 1920,
            fullscreen: Some(FullscreenWrapper::Exclusive),
            last_monitor_x: 0,
            last_monitor_y: 0,
            last_monitor_refresh_rate: 60,
        }
    }
}
