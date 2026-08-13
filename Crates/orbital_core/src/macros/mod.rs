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
            use ::winit::{
                event_loop::EventLoop,
                platform::android::EventLoopBuilderExtAndroid,
            };

            // Set up the cross-platform file manager (AAssetManager + internal
            // storage) before anything tries to load assets.
            let _ = $crate::file_manager::FileManager::init_android_global(
                app.asset_manager(),
                app.internal_data_path(),
            );

            let event_loop = EventLoop::builder().with_android_app(app).build();

            $entrypoint_fn(event_loop);
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
