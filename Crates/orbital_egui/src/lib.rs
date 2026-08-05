//! egui integration for the Orbital engine.
//!
//! Provides an [`EguiModule`] that adds an immediate-mode debug UI overlay.
//! The overlay renders after the main scene pass and can display performance
//! statistics, scene inspection, and pipeline information.
//!
//! # Usage
//!
//! ```ignore
//! use orbital::app::{App, AppSettings};
//! use orbital_egui::EguiModule;
//! use winit::keyboard::KeyCode;
//!
//! App::new()
//!     .add_module(EguiModule::new().with_toggle_key(KeyCode::F2))
//!     .liftoff(event_loop, settings);
//! ```
//!
//! # ECS Integration
//!
//! Panels are stored as an ECS resource ([`EguiPanels`]) rather than on the
//! overlay itself. Any module can register panels without depending on the
//! egui module directly:
//!
//! ```ignore
//! if let Some(mut panels) = ecs.get_resource_mut::<EguiPanels>() {
//!     panels.0.push(Box::new(MyCustomPanel));
//! }
//! ```

mod module;
mod overlay;
mod state;
mod system;
pub mod ui;

pub use egui;
pub use module::EguiModule;
pub use overlay::EguiOverlay;
pub use state::{EguiPanels, EguiState};
