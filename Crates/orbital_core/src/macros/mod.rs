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
                    // winit allows only one EventLoop per process. A previous
                    // loop already exists (e.g. a recreated activity after an
                    // OOM kill), so this process is poisoned. Terminate it so
                    // the next launch starts fresh instead of hanging.
                    ::std::process::exit(0);
                }
            };

            $entrypoint_fn(Ok(event_loop));

            // The event loop only returns when the app exits. Terminate the
            // process so a relaunch starts a fresh process instead of reusing
            // this one (whose EVENT_LOOP_CREATED flag is still set).
            ::std::process::exit(0);
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
