#[macro_export]
macro_rules! make_desktop_main {
    ($entrypoint_fn:ident) => {
        #[cfg(not(target_os = "android"))]
        #[allow(dead_code)]
        fn main() {
            use ::winit::event_loop::EventLoop;

            let event_loop = EventLoop::builder().build();

            $entrypoint_fn(event_loop);
        }
    };
}

#[macro_export]
macro_rules! make_android_main {
    ($entrypoint_fn:ident) => {
        #[cfg(target_os = "android")]
        #[allow(dead_code)]
        #[unsafe(no_mangle)]
        fn android_main(app: ::winit::platform::android::activity::AndroidApp) {
            $crate::logging::init();

            let _ = $crate::file_manager::FileManager::init_android_global(
                app.asset_manager(),
                app.internal_data_path(),
            );

            use ::winit::{
                event_loop::EventLoop,
                platform::android::EventLoopBuilderExtAndroid,
            };

            let event_loop = match EventLoop::builder().with_android_app(app).build() {
                Ok(el) => el,
                Err(e) => {
                    $crate::logging::error!("Event loop build failed: {:?}", e);
                    // winit allows only one EventLoop per process. This
                    // android_main was invoked for a recreated activity in an
                    // already-running process, so there is nothing to do here.
                    // Return (and let this thread end) like Bevy does, leaving
                    // the original event loop alive so the app keeps running
                    // and resumes on the next open.
                    return;
                }
            };

            $entrypoint_fn(Ok(event_loop));

            // The event loop only returns when the app exits. Just return from
            // android_main; do not kill the process.
        }
    };
}

/// Generates both the desktop (`main`) and Android (`android_main`) entry points.
///
/// Each inner macro carries its own `cfg` gate, so exactly one compiles per target:
/// - Desktop: `fn main()`
/// - Android: `#[no_mangle] fn android_main(app: AndroidApp)`
#[macro_export]
macro_rules! make_main {
    ($entrypoint_fn:ident) => {
        $crate::make_desktop_main!($entrypoint_fn);
        $crate::make_android_main!($entrypoint_fn);
    };
}
