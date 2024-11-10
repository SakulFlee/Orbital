use orbital::{
    app::{App, AppRuntime, AppSettings},
    game::{GameRuntime, GameSettings},
    logging,
    renderer::StandardRenderer,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::game::ExampleGame;

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");
    // let settings = GameSettings::default();

    // GameRuntime::<ExampleGame, StandardRenderer>::liftoff(event_loop, settings)
    //     .expect("Runtime failure");

    AppRuntime::<X>::liftoff(event_loop, AppSettings::default()).expect("Runtime failure");
}

pub struct X {}

impl App for X {
    fn init(
        config: &orbital::wgpu::SurfaceConfiguration,
        device: &orbital::wgpu::Device,
        queue: &orbital::wgpu::Queue,
    ) -> Self
    where
        Self: Sized,
    {
        Self {}
    }
}
