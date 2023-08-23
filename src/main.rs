// <<< Imports >>>
use crate::AppWindow;

// <<< Modules >>>
/// Everything the engine needs.
/// This is the rendering (and computation?) backend.
/// The engine will receive an update from `World` and render it.
mod engine;
pub use engine::*;

/// Windowing (if needed; Exception -> WASM).
/// Displays whatever the `Engine` returns onto the screen.
mod app_window;
pub use app_window::*;

/// Game World with all it's objects and states.
mod world;
pub use world::*;

/// Our Application
mod app;
pub use app::*;

// Our Application's configuration
mod app_config;
pub use app_config::*;

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
