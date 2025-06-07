//! ⚠️ You are most likely looking for the [App] description!

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

/// Implement this trait to make an [App].  
/// An [App] is a entrypoint wrapper exposing a few functions for you to use.
/// The main goal of an [App] is to simplify and streamline the process to
/// realization of ideas.
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
/// To use an [App], all you need to do is call the
/// [AppRuntime] with your implementation, like so:
///
/// TODO: New example needed
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
/// Make a structure and implement the trait like so:
///
/// TODO: New example needed
///
/// Each function should be straight forward and easy to understand.
/// Not every function needs to be implemented, many have default
/// implementations which, by default, do nothing.
///
/// [App::init] gets called ONCE at the beginning during [AppRuntime::liftoff].  
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
/// TODO : Needs new example or similar description
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
pub trait App {
    /// Gets called upon the [App] getting resumed _OR_ when the [App] got initiated first time and we know have access to the GPU via [Device] & [Queue].
    /// Depending on the state, we might want to reinitialize things for the GPU related to memory between suspension and resumption might have been dropped.
    fn on_resume(
        &mut self,
        _config: &SurfaceConfiguration,
        _device: &Device,
        _queue: &Queue,
    ) -> impl std::future::Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }

    /// Gets called upon the [App] getting suspended.
    /// On some operating systems this will invalidate the [Device], [Queue], [Surface](wgpu::Surface) and [Window](winit::window::Window).
    fn on_suspend(&mut self) -> impl std::future::Future<Output = ()> + Send
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
    ) -> impl std::future::Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }

    fn on_focus_change(&mut self, _focused: bool) -> impl std::future::Future<Output = ()> + Send
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
    ) -> impl std::future::Future<Output = Option<Vec<AppEvent>>> + Send
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
    ) -> impl std::future::Future<Output = ()> + Send
    where
        Self: Sized,
    {
        async {}
    }
}
