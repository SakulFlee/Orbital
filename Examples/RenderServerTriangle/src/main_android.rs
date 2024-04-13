use crate::entrypoint::entrypoint;
use akimo_runtime::winit::{
    event_loop::EventLoopBuilder,
    platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid},
};

#[no_mangle]
fn android_main(app: AndroidApp) {
    let event_loop = EventLoopBuilder::new().with_android_app(app).build();
    entrypoint(event_loop);
}
