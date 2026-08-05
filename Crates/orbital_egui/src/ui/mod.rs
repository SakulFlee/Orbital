//! UI panels for the egui debug overlay.
//!
//! Each panel is a self-contained widget that can be registered as an
//! ECS resource via [`EguiPanels`](crate::EguiPanels) and is drawn
//! every frame when the overlay is visible.
//!
//! # Built-in Panels
//!
//! - [`performance::PerformancePanel`] — frame time, FPS, GPU timing
//! - [`scene::ScenePanel`] — entity list, component inspection

pub mod performance;
pub mod scene;

use egui::Ui;

/// A panel that can be displayed in the egui overlay.
///
/// Implement this trait to create custom debug panels. Panels are called
/// every frame when the overlay is visible.
pub trait Panel: Send + Sync {
    /// Draw the panel UI using the given egui context.
    fn ui(&mut self, ui: &mut Ui);
}
