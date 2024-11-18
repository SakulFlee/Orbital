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

    AppRuntime::liftoff::<X>(event_loop, AppSettings::default()).expect("Runtime failure");
}

pub struct X {}

impl App for X {
    fn initialize() -> Self
    where
        Self: Sized,
    {
        X {}
    }

    fn on_resume(
        &mut self,
        _config: &orbital::wgpu::SurfaceConfiguration,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) where
        Self: Sized,
    {
        println!("Resumed");
    }
}
