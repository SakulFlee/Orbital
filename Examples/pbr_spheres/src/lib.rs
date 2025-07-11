// use orbital::{
//     app::{AppRuntime, AppSettings},
//     logging::{self, error, info},
//     make_main,
//     winit::{error::EventLoopError, event_loop::EventLoop},
// };
//
// use orbital::app::standard::StandardApp;
//
// mod element;
// use element::*;
//
// pub const NAME: &str = "Orbital-Demo-Project: PBRSpheres";
//
// pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
//     logging::init();
//
//     let event_loop = event_loop_result.expect("Event Loop failure");
//
//     let mut app_settings = AppSettings::default();
//     app_settings.vsync_enabled = false;
//     app_settings.name = NAME.to_string();
//
//     let app = StandardApp::with_initial_elements(vec![Box::new(PBRSpheres)]);
//
//     match AppRuntime::liftoff(event_loop, app_settings, app) {
//         Ok(()) => info!("Cleanly exited!"),
//         Err(e) => error!("Runtime failure: {e:?}"),
//     }
// }

use orbital::{
    logging::{self, info},
    make_android_main,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

pub fn entrypoint(_event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    info!("EMPTY FOR NOW!");
}

make_android_main!(entrypoint);
