use akimo_runtime::{
    logging::init_logger,
    runtime::{Runtime, RuntimeSettings},
};
use app::App;
use log::info;
use winit::{error::EventLoopError, event_loop::EventLoop};

#[cfg(target_os = "android")]
mod main_android;

pub mod app;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    init_logger();

    info!("{:-^1$}", "Clear Screen Example (Akimo-Project)", 80);
    info!("The most simple graphical example, it provides the following:");
    info!("- Initialize and open a Window.");
    info!("  Or on Android: Use the provided AndroidApp instance and");
    info!("  convert it to a window surface we can use.");
    info!("- Initialize a connection to the GPU following the best match.");
    info!("- Setup a color variable (RGB, all f64; 0.0f-1.0f range)");
    info!("- On Update: Increment/Decrement the color variable by an amount");
    info!("- On Render: Create a render pass and make a simple clear screen");
    info!("  operation, using the color variable as the clear screen color.");
    info!("{:-^1$}", "License & Credit", 80);
    info!("Same as this project!");
    info!("{:-^1$}", "", 80);
    info!("");

    let event_loop = event_loop_result.expect("EventLoop building failed!");
    Runtime::liftoff::<App>(event_loop, RuntimeSettings::default()).expect("Runtime failure");
}
