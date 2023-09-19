use wgpu_engine::{log_init, App};

// TODO: Remove Tokio

/// Main function
#[tokio::main(worker_threads = 16)]
async fn main() {
    // Log initialization
    log_init();

    // App
    let app = App::from_app_config_default_path();
    app.hijack_thread_and_run().await;
}
