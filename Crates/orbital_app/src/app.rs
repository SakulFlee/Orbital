//! App builder — combines modules and starts the engine.
//!
//! Usage:
//! ```ignore
//! App::new()
//!     .add_module(MyModule)
//!     .add_module(OtherModule)
//!     .liftoff(event_loop, settings)
//! ```

use orbital_ecs::{System, World};
use wgpu::{Device, Queue};

use crate::{Module, ModuleRuntime};

/// Application builder — the entry point for Orbital applications.
///
/// Collects one or more `Module`s and starts the engine runtime.
/// Inspired by Bevy's `App` builder pattern.
pub struct App {
    modules: Vec<Box<dyn Module>>,
}

impl App {
    /// Create a new empty application.
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    /// Add a module (plugin) to the application.
    ///
    /// Modules contribute ECS entities, resources, and systems.
    /// Multiple modules can be added — their systems are merged
    /// into a single game schedule.
    pub fn add_module<M: Module + 'static>(mut self, module: M) -> Self {
        self.modules.push(Box::new(module));
        self
    }

    /// Start the engine runtime with the configured modules.
    ///
    /// This creates the ECS world, initializes engine resources,
    /// calls each module's `setup()`, and runs the event loop.
    ///
    /// `settings` controls window size, vsync, etc.
    /// `event_loop` is a `winit::event_loop::EventLoop`.
    pub fn liftoff(
        self,
        event_loop: winit::event_loop::EventLoop<()>,
        settings: crate::AppSettings,
    ) -> Result<(), winit::error::EventLoopError> {
        ModuleRuntime::liftoff(event_loop, settings, CombinedModule {
            modules: self.modules,
        })
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

struct CombinedModule {
    modules: Vec<Box<dyn Module>>,
}

impl Module for CombinedModule {
    fn setup(
        &self,
        ecs: &mut World,
        device: &Device,
        queue: &Queue,
    ) -> Vec<Box<dyn System>> {
        let mut all_systems: Vec<Box<dyn System>> = Vec::new();
        for module in &self.modules {
            let systems = module.setup(ecs, device, queue);
            all_systems.extend(systems);
        }
        log::info!("CombinedModule: {} systems from {} modules",
            all_systems.len(), self.modules.len());
        all_systems
    }
}
