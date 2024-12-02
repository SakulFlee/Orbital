use std::time::Duration;

use crate::app::AppSettings;

#[derive(Default, Debug, Clone)]
pub struct GameSettings {// TODO: Cleanup
    pub app_settings: AppSettings,
    pub pipeline_cache: CacheSettings,
    pub material_cache: CacheSettings,
}

#[derive(Debug, Clone)]
pub struct CacheSettings {
    pub cleanup_interval: Duration,
    pub retain_period: Duration,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            cleanup_interval: Duration::from_secs(30),
            retain_period: Duration::from_secs(30),
        }
    }
}
