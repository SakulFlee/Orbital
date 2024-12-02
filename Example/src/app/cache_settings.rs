use std::time::Duration;

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
