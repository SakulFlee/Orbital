// << Imports >>
use app::{app_config::AppConfig, App};

// << Modules >>
pub mod app;
pub mod engine;

// << Constants >>
pub const APP_NAME: &'static str = "WGPU-Engine";

// <<< Functions >>>

/// Main function. ⚠️ Async ⚠️
///
/// Tokio's worker threads can be adjusted if needed.
#[tokio::main(worker_threads = 16)]
async fn main() {
    // Log initialization
    log_init().await;

    // App
    let app = App::from_app_config_default_path().await;
    app.start();
}

async fn log_init() {
    env_logger::init();
    log::info!(
        "Logger initialized at max level set to {}",
        log::max_level()
    );

    log::info!("<<< WGPU-Engine >>>");
}
