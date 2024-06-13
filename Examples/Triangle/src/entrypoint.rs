use akimo_runtime::{
    app::{AppRuntime, AppSettings},
    log,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::app::TriangleApp;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    log::init();

    let event_loop = event_loop_result.expect("Event Loop failure");
    let settings = AppSettings::default();

    AppRuntime::<TriangleApp>::liftoff(event_loop, settings).expect("Runtime failure");
}
