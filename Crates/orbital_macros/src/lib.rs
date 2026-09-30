//! Procedural macros for the Orbital engine.
//!
//! The only export today is [`macro@main`], which replaces Orbital's
//! `make_desktop_main!` / `make_android_main!` / `make_ios_main!` macro trio
//! with a single attribute on the app's entrypoint.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

/// Generates every platform entrypoint from a single app entrypoint.
///
/// Applied to a function taking
/// `Result<orbital::winit::event_loop::EventLoop<()>, orbital::winit::error::EventLoopError>`,
/// this emits a desktop `run()` plus the Android and iOS entrypoints:
///
/// ```ignore
/// #[orbital::main]
/// pub fn entrypoint(
///     event_loop: Result<
///         orbital::winit::event_loop::EventLoop<()>,
///         orbital::winit::error::EventLoopError,
///     >,
/// ) {
///     let event_loop = event_loop.expect("Event Loop failure");
///     // ...
/// }
/// ```
///
/// `src/main.rs` then only needs `fn main() { <crate>::run(); }`.
///
/// The entrypoint receives the result of `EventLoop::builder().build()`
/// unwrapped, so it can report a failed event loop however it likes instead of
/// panicking inside a macro-generated function.
#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let ident = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let attrs = &input.attrs;

    quote! {
        #(#attrs)*
        #vis #sig #block

        /// Builds the event loop and hands it to the entrypoint.
        ///
        /// Call this from `fn main()`; Android and iOS use their own
        /// entrypoints instead.
        #[allow(dead_code)]
        pub fn run() {
            let event_loop = ::orbital::winit::event_loop::EventLoop::builder().build();

            #ident(event_loop);
        }

        #[cfg(target_os = "android")]
        #[allow(dead_code)]
        #[unsafe(no_mangle)]
        fn android_main(app: ::orbital::winit::platform::android::activity::AndroidApp) {
            ::orbital::logging::init();

            let _ = ::orbital::file_manager::FileManager::init_android_global(
                app.asset_manager(),
                app.internal_data_path(),
            );

            use ::orbital::winit::platform::android::EventLoopBuilderExtAndroid;

            let event_loop = match ::orbital::winit::event_loop::EventLoop::builder()
                .with_android_app(app)
                .build()
            {
                Ok(event_loop) => event_loop,
                Err(e) => {
                    ::orbital::logging::error!("Event loop build failed: {:?}", e);

                    // winit allows only one EventLoop per process. This
                    // android_main was invoked for a recreated activity in an
                    // already-running process, so there is nothing to do here.
                    // Return (and let this thread end) like Bevy does, leaving
                    // the original event loop alive so the app keeps running
                    // and resumes on the next open.
                    return;
                }
            };

            #ident(Ok(event_loop));

            // The event loop only returns when the app exits. Just return from
            // android_main; do not kill the process.
        }

        #[cfg(target_os = "ios")]
        #[allow(dead_code)]
        #[unsafe(no_mangle)]
        extern "C" fn ios_main() {
            ::orbital::logging::init();

            let _ = ::orbital::file_manager::FileManager::init_ios_global();

            let event_loop = ::orbital::winit::event_loop::EventLoop::builder().build();

            #ident(event_loop);
        }
    }
    .into()
}
