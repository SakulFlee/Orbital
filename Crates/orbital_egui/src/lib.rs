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

mod overlay;
pub mod ui;

use std::sync::Mutex;

use egui::Context;
use orbital_app::{Module, RenderOverlayResource};
use orbital_ecs::{IntoSystem, Res, ResMut, System, World};
use orbital_ecs_bridge::InputSnapshot;
use orbital_input::InputButton;
use wgpu::{Device, Queue, TextureFormat};
use winit::keyboard::{KeyCode, PhysicalKey};

pub use egui;

// ---------------------------------------------------------------------------
// ECS resources
// ---------------------------------------------------------------------------

/// Core egui state stored as an ECS resource.
///
/// Contains the shared [`Context`], visibility flag, and toggle keybinding.
/// Systems read [`EguiState::enabled`] to check if the UI is visible.
pub struct EguiState {
    pub ctx: egui::Context,
    pub enabled: bool,
    pub(crate) toggle_key: KeyCode,
    pub(crate) was_pressed: bool,
}

impl EguiState {
    /// Returns `true` if egui wants to consume pointer events (clicks, drags).
    pub fn wants_pointer(&self) -> bool {
        self.enabled && self.ctx.egui_wants_pointer_input()
    }

    /// Returns `true` if egui wants to consume keyboard events.
    pub fn wants_keyboard(&self) -> bool {
        self.enabled && self.ctx.egui_wants_keyboard_input()
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

/// Module that adds an egui debug UI overlay to the application.
///
/// # Example
///
/// ```ignore
/// App::new()
///     .add_module(
///         EguiModule::new()
///             .with_toggle_key(KeyCode::F2)
///             .with_panel(ui::performance::PerformancePanel),
///     )
///     .liftoff(event_loop, settings);
/// ```
pub struct EguiModule {
    toggle_key: KeyCode,
    panels: Vec<Box<dyn ui::Panel>>,
}

impl EguiModule {
    pub fn new() -> Self {
        Self {
            toggle_key: KeyCode::F2,
            panels: Vec::new(),
        }
    }

    /// Set the key that toggles the egui overlay. Defaults to F2.
    pub fn with_toggle_key(mut self, key: KeyCode) -> Self {
        self.toggle_key = key;
        self
    }

    /// Add a panel to the egui overlay.
    pub fn with_panel(mut self, panel: impl ui::Panel + 'static) -> Self {
        self.panels.push(Box::new(panel));
        self
    }
}

impl Default for EguiModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for EguiModule {
    fn setup(&self, ecs: &mut World, device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        let format = ecs
            .get_resource::<orbital_ecs_bridge::SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);

        let ctx = Context::default();

        // egui_winit::State requires a &dyn HasDisplayHandle (Window), which we
        // don't have during Module::setup(). The overlay defers State creation
        // to the first frame via the `initialized` flag in on_window_event/render.
        // We store None here and create it when we have access to the window.

        let wgpu_renderer = egui_wgpu::Renderer::new(
            device,
            format,
            egui_wgpu::RendererOptions {
                depth_stencil_format: None,
                ..Default::default()
            },
        );

        // Insert ECS resource for toggle system and wants_pointer/wants_keyboard checks
        ecs.insert_resource(EguiState {
            ctx: ctx.clone(),
            enabled: true,
            toggle_key: self.toggle_key,
            was_pressed: false,
        });

        // Create the overlay
        let panels: Vec<Box<dyn ui::Panel>> = self.panels.iter().map(|p| p.clone_panel()).collect();
        let overlay = overlay::EguiOverlay {
            egui_ctx: ctx,
            winit_state: None, // deferred to first frame
            wgpu_renderer,
            panels,
            initialized: false,
        };

        ecs.insert_resource(RenderOverlayResource(Mutex::new(Box::new(overlay))));

        vec![sys_egui_toggle.into_system()]
    }
}

// ---------------------------------------------------------------------------
// Toggle system
// ---------------------------------------------------------------------------

/// System that toggles the egui overlay on/off when the toggle key is pressed.
pub fn sys_egui_toggle(input: Res<InputSnapshot>, mut state: ResMut<EguiState>) {
    let pressed = input
        .0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(
            state.toggle_key,
        )))
        .map(|(_, s)| s)
        .unwrap_or(false);
    if pressed && !state.was_pressed {
        state.enabled = !state.enabled;
    }
    state.was_pressed = pressed;
}

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use overlay::EguiOverlay;
