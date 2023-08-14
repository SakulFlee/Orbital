use std::sync::Arc;

use wgpu_engine::{Engine, Window};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

const EXAMPLE_NAME: &'static str = "002 - Window Init";

#[cfg_attr(
    all(
        feature = "64-threads",
        not(any(
            feature = "32-threads",
            feature = "16-threads",
            feature = "8-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 64)
)]
#[cfg_attr(
    all(
        feature = "32-threads",
        not(any(
            feature = "64-threads",
            feature = "16-threads",
            feature = "8-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 32)
)]
#[cfg_attr(
    all(
        feature = "16-threads",
        not(any(
            feature = "64-threads",
            feature = "32-threads",
            feature = "8-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 16)
)]
#[cfg_attr(
    all(
        feature = "8-threads",
        not(any(
            feature = "64-threads",
            feature = "32-threads",
            feature = "16-threads",
            feature = "4-threads"
        ))
    ),
    tokio::main(worker_threads = 8)
)]
#[cfg_attr(
    all(
        feature = "4-threads",
        not(any(
            feature = "64-threads",
            feature = "32-threads",
            feature = "16-threads",
            feature = "8-threads",
        ))
    ),
    tokio::main(worker_threads = 4)
)]
#[cfg_attr(
    not(any(
        feature = "64-threads",
        feature = "32-threads",
        feature = "16-threads",
        feature = "8-threads",
        feature = "4-threads",
    )),
    tokio::main(worker_threads = 1)
)]
async fn main() {
    log_init().await;
}

async fn log_init() {
    env_logger::init();
    log::info!(
        "Logger initialized at max level set to {}",
        log::max_level()
    );

    print_thread_feature().await;

    log::info!("001 - Engine Init");
}

async fn print_thread_feature() {
    if cfg!(feature = "64-threads") {
        log::debug!("64-threads: Enabled");
    } else {
        log::debug!("64-threads: Disabled");
    }

    if cfg!(feature = "32-threads") {
        log::debug!("32-threads: Enabled");
    } else {
        log::debug!("32-threads: Disabled");
    }

    if cfg!(feature = "16-threads") {
        log::debug!("16-threads: Enabled");
    } else {
        log::debug!("16-threads: Disabled");
    }

    if cfg!(feature = "8-threads") {
        log::debug!("8-threads: Enabled");
    } else {
        log::debug!("8-threads: Disabled");
    }

    if cfg!(feature = "4-threads") {
        log::debug!("4-threads: Enabled");
    } else {
        log::debug!("4-threads: Disabled");
    }

    if cfg!(not(any(
        feature = "64-threads",
        feature = "32-threads",
        feature = "16-threads",
        feature = "8-threads",
        feature = "4-threads",
    ))) {
        log::debug!("1-threads (Default): Enabled");
    } else {
        log::debug!("1-threads (Default): Disabled");
    }

    let event_loop = EventLoop::new();
    let window = Arc::new(Window::build_and_open(
        EXAMPLE_NAME,
        PhysicalSize::new(1280, 720),
        false,
        true,
        None,
        None,
        None,
        &event_loop,
    ));

    let engine = Engine::initialize(window.clone()).await;
    engine.configure().await;

    let gl_backend = if cfg!(feature = "gl_vulkan") {
        "Vulkan"
    } else if cfg!(feature = "gl_metal") {
        "Metal"
    } else if cfg!(feature = "gl_dx12") {
        "Dx12"
    } else if cfg!(feature = "gl_dx11") {
        "Dx11"
    } else if cfg!(feature = "gl_opengl") {
        "Opengl"
    } else if cfg!(feature = "gl_browser_webgpu") {
        "Browser WebGPU"
    } else {
        "None"
    };
    log::info!("GL Backend: {gl_backend}");

    event_loop.run(move |event, _target, control_flow| {
        // Immediately start a new cycle once a loop is completed.
        // Ideal for games, but more resource intensive.
        *control_flow = ControlFlow::Poll;

        // <<< Events >>>
        match event {
            Event::WindowEvent { window_id, event } => {
                log::debug!("Window Event :: Window ID: {window_id:?}, Event: {event:?}");

                // Validate that the window ID match.
                // Should only be different if multiple windows are used.
                if window_id != window.get_window().id() {
                    log::warn!("Invalid window ID for above's Event!");
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Close requested! Exiting ...");
                        *control_flow = ControlFlow::ExitWithCode(0);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    });
}
