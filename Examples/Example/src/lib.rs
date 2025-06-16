use orbital::{
    app::{AppRuntime, AppSettings},
    logging::{self, error, info},
    make_main,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

pub mod app;
use app::*;
use orbital::app::App;

pub const NAME: &str = "Orbital-Demo-Project";

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = false;
    app_settings.name = NAME.to_string();

    match AppRuntime::liftoff(event_loop, app_settings, MyApp::new()) {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

make_main!(entrypoint);
