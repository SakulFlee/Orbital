//! ⚠️ You are most likely looking for the [Game] description!

pub mod runtime;

pub mod settings;
pub use settings::*;

pub mod world;
pub use world::*;

/// Implement this trait to make a [Game].  
/// A [Game] effectively builds upon an [App], but automates everything
/// much further.
///
/// A [Game] gets presented a [World].  
/// A [World] contains everything inside your [Game].  
/// Things inside your [World] are called [Elements].  
/// An [Element] can receive updates, messages, spawn or despawn models or
/// even other [Elements].  
/// Thus, your [Game] instance is more of a controlling entity:  
/// In most cases it only spawns the initial set of [Elements] and everything
/// else is handled by [Elements] themselves.  
/// You will have **one** [Game], **one** [World] and **many** [Elements].
///
/// You **do not render yourself**!  
/// This is handled by your [World] and [Renderer].
/// The [World] keeps track of any [Models] and informs the [Renderer]
/// about what is to be rendered.
///
/// Additionally, the [GameRuntime] will initialize a few things for you
/// and keep track of them in the background.
/// Such as a frame timer, FPS rating, or, caches for various [resources].
///
/// Once your [Game] instance is ready, continue reading about the
/// [World] and [Elements].
///
/// # Usage
///
/// To use a [Game], all you need to do is call the
/// [GameRuntime] with your implementation, like so:
///
/// ```rust
/// # use orbital::{
/// #     renderer::StandardRenderer, 
/// #     game::{Game, GameRuntime, GameSettings, World}, 
/// #     winit::event_loop::EventLoop
/// # };
///
/// # struct MyGame;
/// # impl Game for MyGame {
/// #   fn init() -> Self
/// #   where Self: Sized,
/// #   {
/// #       Self {}
/// #   }
/// #   fn on_startup(&mut self, _world: &mut World)
/// #   where
/// #       Self: Sized,
/// #   {
/// #       std::process::exit(0);
/// #   }
/// # }
///
/// # fn main() {
/// let event_loop = EventLoop::new().unwrap(); // Acquire event loop, might be different based on platform!
/// let settings = GameSettings::default();
///
/// GameRuntime::<MyGame, StandardRenderer>::liftoff(event_loop, settings)
///     .expect("Runtime failure");
/// # }
/// ```
///
/// You will need three/four things:
///
/// 1. An implementation of [Game]. `MyGame` in this example.
/// 2. An [EventLoop] instance.
/// 3. An [GameSettings] instance.
/// 4. (Optionally) A [Renderer] instance, or, [StandardRenderer].
///
/// ## Making a [Game] instance
///
/// This is basically the exact same procedure as making an [App].  
/// Simply make a structure and implement the trait on it, as needed:
///
/// ```rust
/// # use orbital::{renderer::StandardRenderer, game::Game};
///
/// pub struct MyGame;
///
/// impl Game for MyGame {
/// #   fn init() -> Self
/// #   where Self: Sized,
/// #   {
/// #       Self {}
/// #   }
///     // ...
/// }
/// ```
///
/// Check the function documentation for more information.  
/// However, only [Game::init] **needs** to be implemented.
///
/// ## Acquiring an [EventLoop]
///
/// Actually acquiring a [EventLoop] here is the main challenge.  
/// Depending on your platform(s) choice(s), you may need different entrypoints
/// to handle this per-platform.
///
/// A detailed explanation can be found in the [main crate documentation](crate) under _Platforms_!
///
/// ## Making [GameSettings]
///
/// [GameSettings] actually nest [AppSettings] inside.  
/// It's the same procedure as [AppSettings]:  
/// Initialize it and change values as needed, or use the default
/// implementation.
/// In most cases, the default settings should cover all your needs.
///
/// ## (Optionally) making a [Renderer]
///
/// Lastly, you may need a [Renderer].  
/// This is a trait that, when implemented, will take care of
/// rendering a [World].
/// The internal data stream of what is to be rendered is still
/// handled for you, but you will have to record and submit draw calls
/// yourself.
///
/// This can be useful for debug renders and in case you need a special render
/// like changing the existing [Pipelines].
///
/// Check the [Renderer] trait and [StandardRenderer] for an example.  
/// Support here is limited as there is a virtual infinite amount of
/// possibilities.
/// In most cases, the [StandardRenderer] should be more than enough!
///
/// # Examples
///
/// ## Example project as an example
///
/// Inside the GitHub repository of this project is also a big Example project,
/// which tries to make use of any feature of the engine.
/// The main purpose of this Example is to test engine features in an integrated
/// environment, but also serve as a starting place.
///
/// ## Cubes
///
/// ![Cubes Example GIF](https://raw.githubusercontent.com/SakulFlee/Orbital/main/.github/images/game_example_cubes.gif)
///
/// > ⚠️ The camera movement is done via [TestRenderer](crate::renderer::TestRenderer), this behaviour may change in the future.  
/// > ⚠️ This example does not come with input processing!
///
/// ```rust
/// # use orbital::{
/// #     cgmath::{Deg, Quaternion, Rotation3, Vector3},
/// #     game::{Element, ElementRegistration, Game, World, WorldChange},
/// #     resources::descriptors::{ImportDescriptor, InstanceDescriptor, Instancing, ModelDescriptor},
/// #     ulid::Ulid,
/// # };
///
/// pub struct ExampleGame;
///
/// impl Game for ExampleGame {
///     fn init() -> Self
///     where
///         Self: Sized,
///     {
///         Self {}
///     }
///
///     fn on_startup(&mut self, world: &mut World)
///     where
///         Self: Sized,
///     {
///         // We first need to make an Element.
///         // This is because a Model is associated with a Element ID (ULID).
///         // A Game itself doesn't have such an ID as of now.
///         // Note: Maybe it does in the future! Check the docs and open a PR if
///         // it does and I forgot to update this! <3
///         //
///         // Point being, we need an Element to spawn our Cubes in, so that they
///         // get associated with an Element.
///         // Check the Cube::on_registration function for more!
///         let cube_element = Cube {};
///
///         // Since Elements are dynamic traits, we have to box it
///         // (i.e. place on heap for dynamic access)
///         let boxed_cube = Box::new(cube_element);
///
///         // Finally, we can make the WorldChange and queue the Element spawning.
///         let world_change = WorldChange::SpawnElement(boxed_cube);
///         world.process_world_change(world_change);
///     }
/// }
///
/// #[derive(Debug)]
/// pub struct Cube;
///
/// impl Element for Cube {
///     fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
///         // We can directly supply our ModelDescriptor during registration.
///         // Alternatively, we could queue a WorldChange::SpawnModel(Owned).
///         ElementRegistration {
///             models: Some(vec![ModelDescriptor::FromGLTF( // TODO: Outdated!
///                 "Assets/Models/Cube.glb",
///                 ImportDescriptor::Index(0),
///                 ImportDescriptor::Index(0),
///                 Instancing::Multiple(vec![
///                     // Default
///                     InstanceDescriptor::default(),
///                     // Rotation
///                     InstanceDescriptor {
///                         position: Vector3::new(0.0, -1.0, -1.0),
///                         rotation: Quaternion::from_axis_angle(Vector3::unit_x(), Deg(45.0)),
///                         ..Default::default()
///                     },
///                     InstanceDescriptor {
///                         position: Vector3::new(0.0, -1.0, 0.0),
///                         rotation: Quaternion::from_axis_angle(Vector3::unit_y(), Deg(45.0)),
///                         ..Default::default()
///                     },
///                     InstanceDescriptor {
///                         position: Vector3::new(0.0, -1.0, 1.0),
///                         rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(45.0)),
///                         ..Default::default()
///                     },
///                     // Scale test
///                     InstanceDescriptor {
///                         position: Vector3::new(0.0, 1.0, -1.0),
///                         scale: Vector3::new(2.0, 1.0, 1.0),
///                         ..Default::default()
///                     },
///                     InstanceDescriptor {
///                         position: Vector3::new(0.0, 1.0, 0.0),
///                         scale: Vector3::new(1.0, 2.0, 1.0),
///                         ..Default::default()
///                     },
///                     InstanceDescriptor {
///                         position: Vector3::new(0.0, 1.0, 1.0),
///                         scale: Vector3::new(1.0, 1.0, 2.0),
///                         ..Default::default()
///                     },
///                 ]),
///             )]),
///             ..Default::default()
///         }
///     }
/// }
/// ```
///
/// [App]: crate::app::App
/// [AppSettings]: crate::app::AppSettings
/// [Renderer]: crate::renderer::Renderer
/// [StandardRenderer]: crate::renderer::StandardRenderer
/// [Pipelines]: crate::resources::realizations::Pipeline
/// [EventLoop]: crate::winit::event_loop::EventLoop
/// [resources]: crate::resources
/// [Models]: crate::resources::realizations::Model
/// [Element]: crate::game::world::element::Element
/// [Elements]: crate::game::world::element::Element
pub trait Game { // TODO: Cleanup
    /// Gets called once, upon [GameRuntime::liftoff].  
    /// Any initialization you may need should happen inside here.
    fn init() -> Self
    where
        Self: Sized;

    /// Gets called once, upon startup.  
    /// Any initial [World] changes should happen inside here.
    fn on_startup(&mut self, _world: &mut World)
    where
        Self: Sized,
    {
    }
}
