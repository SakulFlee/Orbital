pub mod app;
pub use app::*;

pub mod engine;
pub use engine::*;

pub mod log;
pub use log::*;

pub mod camera;
pub use camera::*;

pub const APP_NAME: &'static str = "WGPU-Engine";

// TODO: Remove Tokio

/// Main function
#[tokio::main(worker_threads = 16)]
async fn main() {
    // Log initialization
    log::log_init();

    // App
    let app = App::from_app_config_default_path();
    app.hijack_thread_and_run().await;
}
