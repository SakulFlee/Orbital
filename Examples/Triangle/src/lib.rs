use akimo_runtime::{
    logging::init_logger,
    runtime::{Runtime, RuntimeSettings},
};
use app::TriangleApp;
use winit::{error::EventLoopError, event_loop::EventLoop};

#[cfg(target_os = "android")]
mod main_android;

pub mod app;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    init_logger();

    let event_loop = event_loop_result.expect("EventLoop building failed!");
    Runtime::liftoff::<TriangleApp>(event_loop, RuntimeSettings::default())
        .expect("Runtime failure");
}
