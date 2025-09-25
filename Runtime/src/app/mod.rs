//! # Application Module
//!
//! The application module provides the core infrastructure for creating cross-platform
//! applications using the Orbital engine. It handles platform-specific initialization,
//! event loops, and the main application lifecycle.
//!
//! ## Key Components
//!
//! - **App**: The main trait for implementing your application logic
//! - **AppRuntime**: Handles the platform-specific event loop and application lifecycle
//! - **AppSettings**: Configuration for application properties like window size and title
//! - **Input**: Cross-platform input handling system
//!
//! ## Application Lifecycle
//!
//! The application follows a specific lifecycle with events for startup, resume/suspend,
//! resize, update, and render. This allows for proper resource management across different
//! platforms, especially mobile where applications can be suspended and resumed.

use std::future::Future;

use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};

mod settings;
pub use settings::*;

mod runtime_event;
pub use runtime_event::*;

mod runtime;
pub use runtime::*;

mod app_event;
pub use app_event::*;

mod timer;
pub use timer::*;

pub mod input;
use input::*;

pub mod standard;

/// The main application trait that defines the interface between your application
/// and the underlying platform-specific event loop.
/// 
/// Implement this trait to create an [App]. An [App] is an entrypoint wrapper exposing
/// a few functions for you to use. The main goal of an [App] is to simplify and
/// streamline the process to realization of ideas.
///
/// Please note, that an [App] is different from a [Game]!
/// [Apps] are providing you with more control, but you will have to handle more
/// yourself.
/// [Apps] only handle the necessary amount of tasks, like:
/// - Windowing
/// - Event Handling
/// - Providing an easy cross-platform approach to Rust apps
///
/// On the other hand, [Games] automate everything for you,
/// so that you can focus on making [Games] ❤️!
///
/// # Usage
///
/// To use an [App], all you need to do is implement the trait and run it with the
/// appropriate event loop:
///
/// ```rust
/// use orbital::app::App;
/// 
/// struct MyApp;
/// 
/// impl App for MyApp {
///     fn new() -> Self {
///         MyApp
///     }
/// }
/// ```
///
/// You will need three things:
///
/// 1. An implementation of [App]. `MyApp` in this example.
/// 2. An [EventLoop] instance.
/// 3. An [AppSettings] instance.
///
/// ## Making an [App] instance
///
/// Getting an implementation of [App] should be straight forward.
/// Make a structure and implement the trait like in the example above.
///
/// Each function should be straight forward and easy to understand.
/// Not every function needs to be implemented, many have default
/// implementations which, by default, do nothing.
///
/// [App::on_startup] gets called ONCE at the beginning during [AppRuntime::liftoff].  
/// Any other function is **event based**.
/// E.g. [App::on_resize] gets called once there is a resize event.
///
/// ## Acquiring an [EventLoop]
///
/// Actually acquiring a [EventLoop] here is the main challenge.  
/// Depending on your platform(s) choice(s), you may need different entrypoints
/// to handle this per-platform.
///
/// A detailed explanation can be found in the [main crate documentation](crate) under _Platforms_!
///
/// ## Making [AppSettings]
///
/// The [AppSettings] just define a few settings relevant for apps.  
/// Things like window name or initial size can be configured.
/// The default settings are enough to get started.
///
/// [WASM]: https://webassembly.org/
/// [Cargo-APK]: https://github.com/rust-mobile/cargo-apk
/// [Cargo-NDK]: https://github.com/bbqsrc/cargo-ndk
/// [Cargo-NDK-Android-Gradle]: https://github.com/willir/cargo-ndk-android-gradle
/// [xBuild]: https://github.com/rust-mobile/xbuild
/// [EventLoop]: crate::winit::event_loop::EventLoop
/// [EventLoops]: crate::winit::event_loop::EventLoop
/// [Apps]: crate::app::App
/// [Game]: crate::world::Game
/// [Games]: crate::world::Game
/// [GameRuntime]: crate::world::GameRuntime
/// [winit]: crate::winit
pub trait App: Send + Sync {
    /// Creates a new instance of the application.
    /// This is called once at the beginning of the application lifecycle.
    fn new() -> Self;

    /// Called once when the application starts up.
    /// Use this to initialize your application state.
    fn on_startup(&mut self) -> impl Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }

    /// Gets called upon the [App] getting resumed _OR_ when the [App] got initiated first time and we know have access to the GPU via [Device] & [Queue].
    /// Depending on the state, we might want to reinitialize things for the GPU related to memory between suspension and resumption might have been dropped.
    fn on_resume(
        &mut self,
        _config: &SurfaceConfiguration,
        _device: &Device,
        _queue: &Queue,
    ) -> impl Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }

    /// Gets called upon the [App] getting suspended.
    /// On some operating systems this will invalidate the [Device], [Queue], [Surface](wgpu::Surface) and [Window](winit::window::Window).
    fn on_suspend(&mut self) -> impl Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }

    /// Gets called each time the window, app or canvas gets resized.  
    /// Any resizing of resources (e.g. swap-chain, depth texture, etc.) should
    /// be updated inside here.
    fn on_resize(
        &mut self,
        _new_size: cgmath::Vector2<u32>,
        _device: &Device,
        _queue: &Queue,
    ) -> impl Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }

    /// Called when the application focus changes (gains or loses focus).
    fn on_focus_change(&mut self, _focused: bool) -> impl Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }

    /// Gets called each time an update cycle is happening.  
    /// Any updating should happen inside here.
    fn on_update(
        &mut self,
        _input_state: &InputState,
        _delta_time: f64,
        _cycle: Option<(f64, u64)>,
    ) -> impl Future<Output = Option<Vec<AppEvent>>> + Send
    where
        Self: Sized,
    {
        async { None }
    }

    /// Gets called each time a render (== redraw) cycle is happening.
    /// Any rendering should happen inside here.
    fn on_render(
        &mut self,
        _target_view: &TextureView,
        _device: &Device,
        _queue: &Queue,
    ) -> impl Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }
}
