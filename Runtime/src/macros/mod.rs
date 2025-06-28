#[macro_export]
macro_rules! make_main {
    ($entrypoint_fn:ident) => {
        #[cfg(not(target_os = "android"))]
        #[allow(dead_code)]
        fn main() {
            use orbital::winit::event_loop::EventLoop;

            let event_loop = EventLoop::builder().build();

            entrypoint(event_loop);
        }

        #[cfg(target_os = "android")]
        #[allow(dead_code)]
        #[no_mangle]
        fn android_main(app: AndroidApp) {
            use orbital::winit::{
                event_loop::EventLoop,
                platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid},
            };
            let event_loop = EventLoop::builder().with_android_app(app).build();

            entrypoint(event_loop);
        }
    };
}
