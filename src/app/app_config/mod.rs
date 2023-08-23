use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use self::{
    config_adapter::ConfigAdapter, config_monitor::ConfigMonitor, config_window::ConfigWindow,
};

#[cfg(not(debug_assertions))]
use self::wrapper_fullscreen::WrapperFullscreen;

pub mod config_adapter;
pub mod config_monitor;
pub mod config_window;
pub mod wrapper_backend;
pub mod wrapper_fullscreen;

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub window_config: ConfigWindow,
    pub monitor_config: Option<ConfigMonitor>,
    pub adapter_config: Option<ConfigAdapter>,
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
}

impl Default for AppConfig {
    fn default() -> Self {
        #[cfg(not(debug_assertions))]
        {
            Self {
                window_config: Some(ConfigWindow {
                    width: 1920,
                    height: 1080,
                }),
                monitor_config: Some(ConfigMonitor {
                    fullscreen: WrapperFullscreen::Exclusive,
                    position: (0, 0),
                    size: (1920, 1080),
                    refresh_rate: 60,
                }),
                adapter_config: None,
            }
        }
        #[cfg(debug_assertions)]
        {
            Self {
                window_config: ConfigWindow {
                    width: 1280,
                    height: 720,
                },
                monitor_config: None,
                adapter_config: None,
            }
        }
    }
}
