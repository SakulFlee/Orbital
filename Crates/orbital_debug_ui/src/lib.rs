//! In-game debug overlay for Orbital: performance timings, an ECS inspector and
//! a log console, drawn with Iced over the game render.
//!
//! Add [`DebugUiModule`] to your app and press its toggle key (F1 by default) to
//! show or hide the panels.
//!
//! ```ignore
//! App::new()
//!     .add_module(DebugUiModule::new())
//!     .liftoff(event_loop, settings)
//! ```
//!
//! The panels are pure functions of the ECS world, so they always reflect live
//! data and there is no snapshot to keep in sync.

pub mod panels;
pub mod sparkline;
pub mod state;

use iced_wgpu::Renderer as IcedRenderer;
use iced_wgpu::wgpu::{Device, Queue};
use iced_winit::core::Element;
use orbital_app::input::InputButton;
use orbital_app::{LayerRenderer, Module};
use orbital_ecs::{IntoSystem, Res, ResMut, System, World};
use orbital_ecs_bridge::InputSnapshot;
use orbital_iced::{IcedLayerRenderer, IcedState};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::panels::{console_panel, ecs_panel, performance_panel};
use crate::state::{DebugMessage, DebugUiState};

/// Shows or hides the whole overlay when the configured key is pressed.
///
/// Mirrors `orbital_debug_render`'s toggle so the two overlays behave alike.
/// The edge-detection state lives in [`DebugUiState`] because the ECS has no
/// `IntoSystem` impl taking two `ResMut` parameters.
pub fn sys_debug_ui_toggle(input: Res<InputSnapshot>, mut ui: ResMut<DebugUiState>) {
    let pressed = input
        .0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(ui.toggle_key)))
        .map(|(_, pressed)| pressed)
        .unwrap_or(false);

    if pressed && !ui.toggle_was_pressed() {
        ui.toggle_visibility();
        orbital_core::logging::debug!("debug UI {}", if ui.visible { "shown" } else { "hidden" });
    }

    ui.set_toggle_was_pressed(pressed);
}

/// Registers the debug panels with the runtime.
///
/// Debug-only: in a release build the module does nothing, so a shipped game
/// pays neither the build cost nor the per-frame panel cost.
pub struct DebugUiModule {
    toggle_key: KeyCode,
    /// Keeps the panels available in a release build instead of compiling them
    /// out. Off by default; this is a debug tool.
    allow_in_release: bool,
}

impl Default for DebugUiModule {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugUiModule {
    pub fn new() -> Self {
        Self {
            // F1: F3 and F4 are already used by `orbital_debug_render`.
            toggle_key: KeyCode::F1,
            allow_in_release: false,
        }
    }

    pub fn with_toggle_key(mut self, key: KeyCode) -> Self {
        self.toggle_key = key;
        self
    }

    /// Keep the panels available in a release build instead of compiling them
    /// out. Off by default.
    pub fn allow_in_release(mut self, allow: bool) -> Self {
        self.allow_in_release = allow;
        self
    }

    fn is_enabled(&self) -> bool {
        self.allow_in_release || cfg!(debug_assertions)
    }
}

impl Module for DebugUiModule {
    fn setup(&self, ecs: &mut World, _device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        if !self.is_enabled() {
            return vec![];
        }

        ecs.insert_resource(DebugUiState::new(self.toggle_key));

        vec![sys_debug_ui_toggle.into_system()]
    }

    fn register_overlays(
        &self,
        ecs: &mut World,
        layer_renderers: &mut Vec<Box<dyn LayerRenderer>>,
        _legacy_overlays: &mut Vec<Box<dyn orbital_app::RenderOverlay>>,
    ) {
        if !self.is_enabled() {
            return;
        }

        if ecs.get_resource::<DebugUiState>().is_none() {
            orbital_core::logging::warn!(
                "DebugUiModule: DebugUiState missing; register_overlays ran before setup"
            );
            return;
        }

        // Each panel is an `IcedLayerRenderer<DebugMessage>` pushed straight
        // into `layer_renderers` rather than going through `IcedUiState`. That
        // keeps these messages out of the application's own panel messages, and
        // avoids the module-ordering constraint `IcedUiState` implies.
        //
        // The panels are always registered as visible: their own views check
        // `DebugUiState` each frame and render blank when the overlay is
        // hidden. `IcedState::set_visible` cannot be used here, because the
        // toggle lives in the ECS world and the renderer has its own copy of
        // the state.
        // Staggered so the three windows do not overlap on a typical desktop.
        let panels: [(&str, (f32, f32), PanelView); 3] = [
            ("Performance", (60.0, 60.0), performance_panel as PanelView),
            ("ECS", (60.0, 380.0), ecs_panel as PanelView),
            ("Console", (480.0, 60.0), console_panel as PanelView),
        ];

        for (title, position, view) in panels {
            let panel = IcedState::<DebugMessage>::titled(title)
                .with_position(position.0, position.1)
                .with_close_message(DebugMessage::ClosePanel)
                // Rebuilding the tree means re-running the widget diff and
                // layout, and these panels re-read the whole world to do it.
                // At 20 Hz the numbers stay perfectly readable, and
                // `mark_dirty` — which `handle_message` does — bypasses the
                // interval, so a click still shows up immediately.
                .with_refresh_interval(std::time::Duration::from_millis(50))
                .with_view(view);

            layer_renderers.push(Box::new(IcedLayerRenderer::new(panel)));
        }

        orbital_core::logging::debug!("DebugUiModule: registered 3 debug panels");
    }
}

/// A panel view: a pure function from the world to a widget tree.
type PanelView =
    fn(&World) -> Element<'static, DebugMessage, iced_winit::core::Theme, IcedRenderer>;
