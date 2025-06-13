use orbital::{
    app::{AppRuntime, AppSettings},
    logging,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::app::MyApp;

pub const NAME: &str = "Orbital-Demo-Project";

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = false;
    app_settings.name = NAME.to_string();

    AppRuntime::<MyApp>::liftoff(event_loop, app_settings).expect("Runtime invocation failure");
}
