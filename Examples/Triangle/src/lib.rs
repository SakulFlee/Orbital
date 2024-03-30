use akimo_runtime::{
    logging::init_logger,
    runtime::{Runtime, RuntimeSettings},
};
use app::TriangleApp;
use log::info;
use winit::{error::EventLoopError, event_loop::EventLoop};

#[cfg(target_os = "android")]
mod main_android;

pub mod app;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    init_logger();

    info!("{:-^1$}", "Triangle Example (Akimo-Project)", 80);
    info!("Slightly more complex than the Clear Screen example.");
    info!("It does everything (but the color variable!) as the clear");
    info!("screen example does! Start there if you are curious about the");
    info!("minimal required steps for a graphical application.");
    info!("In addition, this does:");
    info!("- Create and compile a shader module (Vertex & Fragment)");
    info!("- Create a pipeline, utilizing the shaders");
    info!("- On Render: Render a single triangle");
    info!("  Each vertex of the triangle will be set to either red,");
    info!("  green, or, blue. Creating a very colorful RGB triangle.");
    info!("{:-^1$}", "License & Credit", 80);
    info!("Same as this project!");
    info!("{:-^1$}", "", 80);
    info!("");

    let event_loop = event_loop_result.expect("EventLoop building failed!");
    Runtime::liftoff::<TriangleApp>(event_loop, RuntimeSettings::default())
        .expect("Runtime failure");
}
