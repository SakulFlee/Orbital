use akimo_runtime::{
    game::GameRuntime,
    log,
    runtime::RuntimeSettings,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::app::ExampleGame;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    log::init();

    let event_loop = event_loop_result.expect("Event Loop failure");
    let settings = RuntimeSettings::default();

    GameRuntime::<ExampleGame>::liftoff(event_loop, settings).expect("Runtime failure");
}
