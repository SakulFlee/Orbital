use akimo_runtime::{
    error::RuntimeError,
    logging::init_logger,
    runtime::{Runtime, RuntimeSettings},
};
use app::App;
use winit::{
    error::EventLoopError,
    event_loop::{EventLoop, EventLoopBuilder},
};

pub mod app;

fn _main(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    init_logger();

    let event_loop = event_loop_result.expect("EventLoop building failed!");
    Runtime::liftoff::<App>(event_loop, RuntimeSettings::default()).expect("Runtime failure");
}

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[allow(dead_code)]
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    let event_loop = EventLoopBuilder::new().with_android_app(app).build();
    _main(event_loop);
}

#[allow(dead_code)]
#[cfg(not(target_os = "android"))]
fn main() {
    let event_loop = EventLoopBuilder::new().build();
    _main(event_loop);
}
