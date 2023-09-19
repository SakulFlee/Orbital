use wgpu_engine::{log_init, WGPURenderingEngine};
use winit::{event_loop::EventLoop, window::WindowBuilder};

#[cfg(test)]
mod tests;

fn main() {
    // Log initialization
    log_init();

    // App
    // let app = App::from_app_config_default_path();
    // app.hijack_thread_and_run().await;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Window creation failed");

    let engine = WGPURenderingEngine::new(&window).expect("Engine creation failed");
}
