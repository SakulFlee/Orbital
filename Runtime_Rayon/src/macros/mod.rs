//! # Macros Module
//!
//! The macros module provides utility macros that simplify common tasks in the Orbital engine,
//! particularly related to application entry points across different platforms.
//!
//! ## Key Macros
//!
//! - **make_desktop_main**: Creates a standard main function for desktop platforms
//! - **make_android_main**: Creates the Android-specific main function for mobile deployment

#[macro_export]
/// Creates a standard main function for desktop platforms that sets up the event loop
/// and calls the specified entrypoint function. This macro is only active on non-Android platforms.
macro_rules! make_desktop_main {
    ($entrypoint_fn:ident) => {
        #[cfg(not(target_os = "android"))]
        #[allow(dead_code)]
        fn main() {
            use orbital::winit::event_loop::EventLoop;

            let event_loop = EventLoop::builder().build();

            entrypoint(event_loop);
        }
    };
}

#[macro_export]
/// Creates the Android-specific main function that sets up the Android application
/// environment and calls the specified entrypoint function. This macro is only active on Android.
macro_rules! make_android_main {
    ($entrypoint_fn:ident) => {
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
