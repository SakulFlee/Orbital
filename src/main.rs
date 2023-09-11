use app::App;

pub mod app;
pub mod engine;
pub mod log;

pub const APP_NAME: &'static str = "WGPU-Engine";

/// Main function
#[tokio::main(worker_threads = 16)]
async fn main() {
    // Log initialization
    log::log_init();

    // App
    let app = App::from_app_config_default_path().await;
    app.hijack_thread_and_run().await;
}
