/// Everything the engine needs.
/// This is the rendering (and computation?) backend.
/// The engine will receive an update from `World` and render it.
mod engine;
pub use engine::*;

/// Windowing (if needed; Exception -> WASM).
/// Displays whatever the `Engine` returns onto the screen.
mod window;
pub use window::*;

/// Game World with all it's objects and states.
mod world;
pub use world::*;
