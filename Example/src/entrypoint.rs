use orbital::{
    app::{AppRuntime, AppSettings},
    logging,
    renderer::StandardRenderer,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use crate::app::{CacheSettings, MyApp};

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let app_settings = AppSettings::default();

    let app = MyApp::<StandardRenderer>::new(CacheSettings::default(), CacheSettings::default());

    AppRuntime::liftoff(event_loop, app_settings, app).expect("Runtime failure");
}
