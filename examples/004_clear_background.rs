use std::{sync::Arc, time::Instant};

use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureViewDescriptor,
};
use wgpu_engine::{Engine, Window};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

const EXAMPLE_NAME: &'static str = "004 - Clear Background";

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

    log::info!("{EXAMPLE_NAME}");
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

    // Colour for clearing
    let mut clear_colour = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let mut clear_colour_index = 0;
    let mut clear_colour_increasing = true;

    // How many cycles have been completed
    let mut cycle_count = 0;
    // How much time has passed since the last cycle
    let mut cycle_time = Instant::now();
    // Delta difference between "now" and "then" per cycle
    let mut delta_time = 0.0;

    event_loop.run(move |event, _target, control_flow| {
        // Immediately start a new cycle once a loop is completed.
        // Ideal for games, but more resource intensive.
        *control_flow = ControlFlow::Poll;

        // <<< Cycle Calculation >>>
        // Increase cycle count and take "now time"
        cycle_count += 1;
        let now_time = Instant::now();
        // Calculate duration since last cycle time
        let duration = now_time.duration_since(cycle_time);
        // Add difference to delta time
        delta_time = delta_time + duration.as_secs_f64();

        // If delta time is over a second, end the cycle
        if delta_time >= 1.0 {
            // Update performance outputs
            let performance_str = format!("UPS: {cycle_count}/s (Δ{delta_time}s)");
            log::debug!("{performance_str}");
            window.get_window().set_title(&format!(
                "{EXAMPLE_NAME} - @{gl_backend} - {performance_str}"
            ));

            // One second has past, subtract that
            delta_time -= 1.0;
            // Reset cycle
            cycle_count = 0;
        }
        // Update cycle time with now time
        cycle_time = now_time;

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
            Event::RedrawRequested(window_id) => {
                log::debug!("Redraw Requested :: Window ID: {window_id:?}");

                // Validate that the window ID match.
                // Should only be different if multiple windows are used.
                if window_id != window.get_window().id() {
                    log::warn!("Invalid window ID for above's Event!");
                    return;
                }

                // TODO: Rendering goes here
                let output_surface_texture = engine
                    .get_surface()
                    .get_current_texture()
                    .expect("failed acquiring current texture of target window");

                let output_surface_texture_view = output_surface_texture
                    .texture
                    .create_view(&TextureViewDescriptor::default());

                let mut command_encoder =
                    engine
                        .get_device()
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                command_encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &output_surface_texture_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(clear_colour),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                let command_buffer = command_encoder.finish();

                // Submit command buffer
                engine.get_queue().submit(vec![command_buffer]);
                output_surface_texture.present();
            }
            Event::RedrawEventsCleared => {
                // Update colour variable
                const INCREASE_RATE: f64 = 0.005;
                match clear_colour_index {
                    0 => {
                        if clear_colour_increasing {
                            clear_colour.r += INCREASE_RATE;
                        } else {
                            clear_colour.r -= INCREASE_RATE;
                        }

                        if clear_colour.r >= 1.0 || clear_colour.r <= 0.0 {
                            clear_colour_increasing = !clear_colour_increasing;
                        }

                        if clear_colour.r <= 0.1 && !clear_colour_increasing {
                            clear_colour_index = 1;
                            clear_colour_increasing = true;
                            clear_colour.r = 0.0;
                        }
                    }
                    1 => {
                        if clear_colour_increasing {
                            clear_colour.g += INCREASE_RATE;
                        } else {
                            clear_colour.g -= INCREASE_RATE;
                        }

                        if clear_colour.g >= 1.0 || clear_colour.g <= 0.0 {
                            clear_colour_increasing = !clear_colour_increasing;
                        }

                        if clear_colour.g <= 0.1 && !clear_colour_increasing {
                            clear_colour_index = 2;
                            clear_colour_increasing = true;
                            clear_colour.g = 0.0;
                        }
                    }
                    2 => {
                        if clear_colour_increasing {
                            clear_colour.b += INCREASE_RATE;
                        } else {
                            clear_colour.b -= INCREASE_RATE;
                        }

                        if clear_colour.b >= 1.0 || clear_colour.b <= 0.0 {
                            clear_colour_increasing = !clear_colour_increasing;
                        }

                        if clear_colour.b <= 0.1 && !clear_colour_increasing {
                            clear_colour_index = 0;
                            clear_colour_increasing = true;
                            clear_colour.b = 0.0;
                        }
                    }
                    _ => (),
                }

                // If redrawing finished -> request to redraw next cycle
                window.get_window().request_redraw();
            }
            _ => (),
        }
    });
}
