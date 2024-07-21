use orbital::{
    game::{GameRuntime, GameSettings},
    logging,
    renderer::StandardRenderer,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::app::ExampleGame;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");
    let settings = GameSettings::default();

    GameRuntime::<ExampleGame, StandardRenderer>::liftoff(event_loop, settings)
        .expect("Runtime failure");
}
