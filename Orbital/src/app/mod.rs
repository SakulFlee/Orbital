//! ⚠️ You are most likely looking for the [App] description!

use wgpu::{Device, Queue, SurfaceConfiguration, TextureView};

pub mod settings;
pub use settings::*;

pub mod event;
pub use event::*;

pub mod runtime;
pub use runtime::*;

pub mod app_changes;
pub use app_changes::*;

use crate::{input::InputState, world::Message};

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
/// ```rust
/// # use orbital::{app::{AppSettings, AppRuntime, App}, wgpu::{Device, Queue, SurfaceConfiguration}, winit::event_loop::EventLoop};
///
/// # struct MyApp;
/// # impl App for MyApp {
/// #   fn init(config: &SurfaceConfiguration, device: &Device, queue: &Queue) -> Self
/// #   where Self: Sized,
/// #   {
/// #       std::process::exit(0);
/// #       Self {}
/// #   }
/// # }
///
/// # fn main() {
/// let event_loop = EventLoop::new().unwrap(); // Acquire event loop, might be different based on platform!
/// let settings = AppSettings::default();
///
/// AppRuntime::<MyApp>::liftoff(event_loop, settings)
///     .expect("Runtime failure");
/// # }
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
/// Make a structure and implement the trait like so:
///
/// ```rust
/// # use orbital::{
/// #     app::{App, AppChange},
/// #     wgpu::{Device, Queue, SurfaceConfiguration}
/// # };
///
/// pub struct MyApp;
///
/// impl App for MyApp {
///     // ...
/// #    fn init(config: &SurfaceConfiguration, device: &Device, _queue: &Queue) -> Self
/// #    where
/// #        Self: Sized,
/// #    {
/// #       std::process::exit(0);
/// #       Self {}
/// #    }
/// }
/// ```
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
/// # Examples
///
/// ## [Game] & [GameRuntime] as an example
/// For a fully integrated example take a look at [Game]
/// and [GameRuntime].
/// Both build on top of an [App].
///
/// ## RGB Triangle with Shader
///
/// ![Example image](https://raw.githubusercontent.com/SakulFlee/Orbital/main/.github/images/app_example_triangle.png)
///
/// ```rust
/// // app.rs
/// use orbital::{
///     app::{App, AppChange},
///     wgpu::{
///         Color, CommandEncoderDescriptor, Device, FragmentState, LoadOp, MultisampleState,
///         Operations, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
///         RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
///         ShaderSource, StoreOp, SurfaceConfiguration, TextureView, VertexState,
///     },
/// };
///
/// pub struct TriangleApp {
///     // Note: Curiously, the following two variables don't have to be stored if
///     // we are just referencing them. Uncomment the below and the end of
///     // Self::init if you need access to them :)
///     //
///     // shader: ShaderModule,
///     // pipeline_layout: PipelineLayout,
///     pipeline: RenderPipeline,
/// }
///
/// impl App for TriangleApp {
///     fn init(config: &SurfaceConfiguration, device: &Device, _queue: &Queue) -> Self
///     where
///         Self: Sized,
///     {
///         # std::process::exit(0);
///         let shader = device.create_shader_module(ShaderModuleDescriptor {
///             label: None,
///             source: ShaderSource::Wgsl(include_str!("your_shader_here.wgsl").into()),
///         });
///
///         let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
///             label: None,
///             bind_group_layouts: &[],
///             push_constant_ranges: &[],
///         });
///
///         let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
///             label: None,
///             layout: Some(&pipeline_layout),
///             vertex: VertexState {
///                 module: &shader,
///                 entry_point: "vs_main",
///                 buffers: &[],
///                 compilation_options: Default::default(),
///             },
///             fragment: Some(FragmentState {
///                 module: &shader,
///                 entry_point: "fs_main",
///                 targets: &[Some(config.format.into())],
///                 compilation_options: Default::default(),
///             }),
///             primitive: PrimitiveState::default(),
///             depth_stencil: None,
///             multisample: MultisampleState::default(),
///             multiview: None,
///         });
///
///         Self {
///             // Note: Check variable description in struct declaration!
///             //
///             // shader,
///             // pipeline_layout,
///             pipeline,
///         }
///     }
///
///     fn on_update(&mut self) -> Option<Vec<AppChange>> {
///         // Nothing needed for this example!
///         // All events that we care about are already taken care of.
///
///         None
///     }
///
///     fn on_render(&mut self, view: &TextureView, device: &Device, queue: &Queue) {
///         let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
///
///         {
///             let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
///                 label: None,
///                 color_attachments: &[Some(RenderPassColorAttachment {
///                     view,
///                     resolve_target: None,
///                     ops: Operations {
///                         load: LoadOp::Clear(Color::BLACK),
///                         store: StoreOp::Store,
///                     },
///                 })],
///                 depth_stencil_attachment: None,
///                 timestamp_writes: None,
///                 occlusion_query_set: None,
///             });
///             render_pass.set_pipeline(&self.pipeline);
///             render_pass.draw(0..3, 0..1);
///         }
///
///         queue.submit(Some(encoder.finish()));
///     }
/// }
/// ```
///
/// ```wgsl
/// // shader.wgsl
/// struct VertexOutput {
///     @builtin(position) clip_position: vec4<f32>,
///     @location(0) color: vec4<f32>,
/// };
///
/// @vertex
/// fn vs_main(
///     @builtin(vertex_index) in_vertex_index: u32
/// ) -> VertexOutput {
///     var out: VertexOutput;
///
///     let x = f32(1 - i32(in_vertex_index)) * 0.5;
///     let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
///
///     out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
///
///     if in_vertex_index == 0 {
///         out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
///     } else if in_vertex_index == 1 {
///         out.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
///     } else if in_vertex_index == 2 {
///         out.color = vec4<f32>(0.0, 0.0, 1.0, 1.0);
///     }
///
///     return out;
/// }
///
/// @fragment
/// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
///     return in.color;
/// }
/// ```
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
        _messages: Vec<Message>,
    ) -> impl std::future::Future<Output = Option<Vec<AppChange>>> + Send
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
