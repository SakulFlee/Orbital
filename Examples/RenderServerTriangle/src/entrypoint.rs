use akimo_runtime::{
    log,
    runtime::{Runtime, RuntimeSettings},
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::app::RenderServerTriangleApp;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    log::init();

    let event_loop = event_loop_result.expect("Event Loop failure");
    let settings = RuntimeSettings::default();

    Runtime::liftoff::<RenderServerTriangleApp>(event_loop, settings).expect("Runtime failure");
}
