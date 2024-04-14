use akimo_runtime::{
    log,
    runtime::{Runtime, RuntimeSettings},
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::app::TriangleApp;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    log::init();

    let event_loop = event_loop_result.expect("Event Loop failure");
    let settings = RuntimeSettings::default();

    Runtime::liftoff::<TriangleApp>(event_loop, settings).expect("Runtime failure");
}
