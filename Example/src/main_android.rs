use crate::entrypoint::entrypoint;
use orbital::winit::{
    event_loop::EventLoop,
    platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid},
};

#[no_mangle]
fn android_main(app: AndroidApp) {
    let event_loop = EventLoop::builder().with_android_app(app).build();

    entrypoint(event_loop);
}
