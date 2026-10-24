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

pub mod editable;
pub mod panels;
pub mod sparkline;
pub mod state;

use iced_wgpu::Renderer as IcedRenderer;
use iced_wgpu::wgpu::{Device, Queue};
use iced_winit::core::Element;
use orbital_app::input::InputButton;
use orbital_app::{LayerRenderer, Module};
use orbital_ecs::{IntoSystem, Res, ResMut, System, World};
use orbital_ecs_bridge::{InputSnapshot, TimingsHistoryConfig};
use orbital_iced::{IcedLayerRenderer, IcedState};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::panels::{console_panel, ecs_panel, edit, performance_panel};
use crate::state::{DebugMessage, DebugUiState, PanelId};

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

/// Keeps the frame-timing history config in sync with the Performance panel.
///
/// Only the visible Performance window needs graph/table history; when it is
/// hidden, `FrameTimings` clears the retained samples.
pub fn sys_sync_timings_history(
    state: Res<DebugUiState>,
    mut config: ResMut<TimingsHistoryConfig>,
) {
    config.enabled = state.is_panel_visible(PanelId::Performance);
    config.window_secs = state.graph_window_secs as f64;
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

        if ecs.get_resource::<TimingsHistoryConfig>().is_none() {
            ecs.insert_resource(TimingsHistoryConfig::default());
        }

        vec![
            sys_debug_ui_toggle.into_system(),
            sys_sync_timings_history.into_system(),
        ]
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

        crate::editable::register_builtin_editable_fields();

        // Each panel is an `IcedLayerRenderer<DebugMessage>` pushed straight
        // into `layer_renderers` rather than going through `IcedUiState`. That
        // keeps these messages out of the application's own panel messages, and
        // avoids the module-ordering constraint `IcedUiState` implies.
        //
        // The toggle lives in the ECS world, so visibility is supplied as a
        // predicate evaluated against the world when the view is built. That
        // hides the whole panel — title bar included — instead of blanking only
        // its content, which used to leave the titles on screen while the
        // overlay was off.
        //
        // Tall list panels get an explicit size so they stay usable next to
        // each other; the windows are draggable and resizable, so this is only
        // a starting point. The console is deliberately wide and short, since
        // log lines are long.
        let panels: [PanelSpec; 3] = [
            (
                "Performance",
                (40.0, 40.0),
                (440.0, 340.0),
                PanelId::Performance,
                performance_panel as PanelView,
            ),
            (
                "ECS",
                (40.0, 420.0),
                (600.0, 460.0),
                PanelId::Ecs,
                ecs_panel as PanelView,
            ),
            (
                "Console",
                (500.0, 40.0),
                (760.0, 260.0),
                PanelId::Console,
                console_panel as PanelView,
            ),
        ];

        for (title, position, size, id, view) in panels {
            let panel = IcedState::<DebugMessage>::titled(title)
                .with_position(position.0, position.1)
                .with_size(size.0, size.1)
                .with_close_message(DebugMessage::HidePanel(id))
                // `handle_message` is the only place holding both the world and
                // the panel state, so it is where a component edit is written.
                .with_message_handler(handle_debug_message)
                // Rebuilding the tree means re-running the widget diff and
                // layout, and these panels re-read the whole world to do it.
                // At 20 Hz the numbers stay perfectly readable, and
                // `mark_dirty` — which `handle_message` does — bypasses the
                // interval, so a click still shows up immediately.
                .with_refresh_interval(std::time::Duration::from_millis(50))
                .with_visibility(move |world: &World| {
                    world
                        .get_resource::<DebugUiState>()
                        .is_some_and(|state| state.is_panel_visible(id))
                })
                .with_view(view);

            layer_renderers.push(Box::new(IcedLayerRenderer::new(panel)));
        }

        orbital_core::logging::debug!("DebugUiModule: registered 3 debug panels");
    }
}

/// Applies a debug panel's message to the world.
///
/// Everything a panel changes lives in the `DebugUiState` resource, so this
/// routes the message there and performs any component write.
fn handle_debug_message(
    _panel: &mut IcedState<DebugMessage>,
    message: DebugMessage,
    world: &mut World,
) {
    if let DebugMessage::ApplyField(ref key) = message {
        let input = world
            .get_resource::<DebugUiState>()
            .and_then(|state| state.field_buffers.get(key).cloned())
            .unwrap_or_default();

        let result = edit::apply_edit(world, key, &input);
        let failure = result.as_ref().err().cloned();

        if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
            state.last_error = failure;
            // Drop the buffer so the field falls back to the component's value.
            state.field_buffers.remove(key);
            state.dirty = true;
        }

        match result {
            Ok(()) => orbital_core::logging::debug!(
                "debug UI: applied field on entity {}",
                key.entity.index
            ),
            Err(error) => orbital_core::logging::warn!("debug UI edit failed: {error}"),
        }

        return;
    }

    match message {
        DebugMessage::AddModelInstance(entity) => {
            edit::add_model_instance(world, entity);
            if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
                state.dirty = true;
            }
            return;
        }
        DebugMessage::RemoveModelInstance { entity, ulid } => {
            edit::remove_model_instance(world, entity, ulid);
            if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
                state.dirty = true;
            }
            return;
        }
        _ => {}
    }

    if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
        state.apply(&message);
    }
}

/// A panel view: a pure function from the world to a widget tree.
type PanelView =
    fn(&World) -> Element<'static, DebugMessage, iced_winit::core::Theme, IcedRenderer>;

/// A panel's title, initial position, initial size, identity and view.
type PanelSpec = (
    &'static str,
    (f32, f32),
    (f32, f32),
    crate::state::PanelId,
    PanelView,
);
