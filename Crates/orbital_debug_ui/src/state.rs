//! Shared UI state and message type for the debug panels.

use std::collections::{BTreeSet, HashSet};

use orbital_ecs::Entity;
use orbital_ecs_bridge::{TIMING_STAGES, TOTAL_STAGE_INDEX};

/// The Iced renderer type the panels draw with.
///
/// Aliased so panel signatures do not have to spell out the whole
/// `iced_wgpu::Renderer` path.
pub type Renderer = iced_wgpu::Renderer;

/// A message published by one of the debug panels.
///
/// Panels are registered as `IcedLayerRenderer<DebugMessage>` directly rather
/// than through `IcedUiState`, so this enum is private to this crate and does
/// not constrain the application's own panel messages.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugMessage {
    /// Focus a different bottom-panel tab.
    SetTab(usize),
    /// Select an entity in the ECS tree, or clear the selection with `None`.
    SelectEntity(Option<Entity>),
    /// Expand or collapse the component list of an entity.
    ToggleEntity(Entity),
    /// Show only entities whose index, label or component names match.
    SetFilter(String),
    /// Add or remove a series from the performance graph list.
    ToggleGraphSeries(GraphSeries),
    /// Choose which statistic the performance graphs plot.
    SetGraphMetric(GraphMetric),
    /// Set the graph history window from the slider.
    SetGraphWindow(f32),
    /// Update the exact graph window text without applying it yet.
    EditGraphWindow(String),
    /// Apply the graph window text field.
    ApplyGraphWindow,
    /// Show only log records at or above this level.
    SetMinLevel(log::LevelFilter),
    /// Drop the captured log buffer.
    ClearLog,
    /// The text of an editable field changed but has not been applied yet.
    EditField(crate::panels::edit::FieldKey, String),
    /// Apply the field's buffered text.
    ///
    /// The text is not carried in the message: pressing Enter in a `text_input`
    /// and pressing the `set` button both mean "apply what is in the buffer",
    /// and `text_input::on_submit` takes a fixed message rather than a closure,
    /// so one message serves both.
    ApplyField(crate::panels::edit::FieldKey),
    /// Add an instance to the selected entity's `ModelInstances`.
    AddModelInstance(Entity),
    /// Remove one model instance by its ULID's integer representation.
    RemoveModelInstance { entity: Entity, ulid: u128 },
    /// Flip a panel's visibility.
    TogglePanel(PanelId),
    /// Close one panel's window.
    HidePanel(PanelId),
}

/// The shortest selectable graph history window, in seconds.
pub const MIN_GRAPH_WINDOW_SECS: f32 = 0.5;
/// The longest selectable graph history window, in seconds.
pub const MAX_GRAPH_WINDOW_SECS: f32 = 60.0;
/// The default graph history window, in seconds.
pub const DEFAULT_GRAPH_WINDOW_SECS: f32 = 10.0;

/// Formats seconds for the slider label / numeric input.
pub fn format_seconds(value: f32) -> String {
    let mut text = format!("{value:.2}");

    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }

    text
}

/// A series that can be plotted in the performance panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GraphSeries {
    /// One of the per-stage timings.
    Stage(usize),
    /// Instantaneous frames per second, derived from frame time.
    Fps,
    /// Wall-clock frame delta in milliseconds.
    Cdt,
}

/// Which statistic of a timing stage the performance graphs plot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GraphMetric {
    /// The most recent frame's duration.
    #[default]
    Last,
    /// The mean over a short rolling window.
    Avg,
    /// The smallest value in a short rolling window.
    Min,
    /// The largest value in a short rolling window.
    Max,
    /// Last, rolling average and the rolling min/max band together.
    All,
}

/// Identifies one of the debug panels, for show/hide and focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelId {
    Performance,
    Ecs,
    Console,
}

/// How many characters of a component's `Debug` output to show before
/// truncating.
///
/// A single component can hold a megabyte of data, and the tree is rebuilt
/// every frame, so unbounded formatting would stall the render loop.
pub const VALUE_PREVIEW_CHARS: usize = 200;

/// At most this many rows are built per frame across all panels.
///
/// The ECS world can hold hundreds of thousands of entities; materialising a
/// row for each would make the panel unusable long before the list was even
/// scrolled.
pub const MAX_ROWS: usize = 200;

/// All debug-UI state, held as an ECS resource.
///
/// The Iced bridge rebuilds each panel's widget tree every frame, so this is
/// where a panel's own state has to live — there is no persistent program.
#[derive(Debug, Clone)]
pub struct DebugUiState {
    /// Whether any debug panel is being shown at all.
    pub visible: bool,
    /// Per-panel visibility, for panels the user has dismissed.
    pub panels: [bool; 3],
    /// Which bottom-panel tab is showing.
    pub active_tab: usize,
    /// The entity whose components are listed in the ECS panel.
    pub selected: Option<Entity>,
    /// Entities whose component list is expanded.
    pub expanded: HashSet<Entity>,
    /// Entity-tree filter; matches the entity index, label or component type
    /// name.
    pub filter: String,
    /// The series whose graphs are shown. Empty means no graph is pinned.
    pub graph_series: BTreeSet<GraphSeries>,
    /// Which statistic those graphs plot.
    pub graph_metric: GraphMetric,
    /// How many seconds of graph history to retain, in seconds.
    pub graph_window_secs: f32,
    /// Uncommitted exact text for the graph window.
    pub graph_window_input: String,
    /// Invalid graph window input, if any.
    pub graph_window_error: Option<String>,
    /// Log-level filter for the console.
    pub min_level: log::LevelFilter,
    /// Uncommitted text for each editable field, keyed by entity, type and
    /// field id.
    ///
    /// Kept in the resource rather than in a widget so the field survives the
    /// panel's rebuild throttling.
    pub field_buffers: std::collections::HashMap<crate::panels::edit::FieldKey, String>,
    /// The message from the last failed edit, shown under the components.
    pub last_error: Option<String>,
    /// Set whenever something changed that the panels should redraw for.
    ///
    /// The per-panel rebuild cadence lives in `orbital_iced::IcedState`; this
    /// flag is the debug state's single place to record that an interaction
    /// changed something.
    pub dirty: bool,
    /// The toggle key.
    pub toggle_key: winit::keyboard::KeyCode,
    /// Whether the toggle key was down on the previous frame, for edge
    /// detection.
    ///
    /// Lives here rather than in a separate resource because the ECS has no
    /// `IntoSystem` impl taking two `ResMut` parameters, so a toggle system
    /// cannot update two resources at once.
    toggle_was_pressed: bool,
}

impl DebugUiState {
    /// A fresh state with every panel visible and nothing selected.
    pub fn new(toggle_key: winit::keyboard::KeyCode) -> Self {
        Self {
            visible: false,
            panels: [true; 3],
            active_tab: 0,
            selected: None,
            expanded: HashSet::new(),
            filter: String::new(),
            graph_series: BTreeSet::from([GraphSeries::Stage(TOTAL_STAGE_INDEX)]),
            graph_metric: GraphMetric::Last,
            graph_window_secs: DEFAULT_GRAPH_WINDOW_SECS,
            graph_window_input: format_seconds(DEFAULT_GRAPH_WINDOW_SECS),
            graph_window_error: None,
            min_level: log::LevelFilter::Info,
            field_buffers: std::collections::HashMap::new(),
            last_error: None,
            dirty: true,
            toggle_key,
            toggle_was_pressed: false,
        }
    }

    fn set_graph_window(&mut self, seconds: f32) {
        let seconds = seconds.clamp(MIN_GRAPH_WINDOW_SECS, MAX_GRAPH_WINDOW_SECS);
        self.graph_window_secs = seconds;
        self.graph_window_input = format_seconds(seconds);
        self.graph_window_error = None;
        self.dirty = true;
    }

    /// The uncommitted text for an editable field.
    ///
    /// `current` is the component's present value, used until the user types
    /// something, so the field is pre-filled rather than blank. Kept in the
    /// resource so the text survives the panel's rebuild throttling.
    pub fn field_buffer(&self, key: &crate::panels::edit::FieldKey, current: &str) -> String {
        self.field_buffers
            .get(key)
            .cloned()
            .unwrap_or_else(|| current.to_string())
    }

    /// Whether `panel` should be drawn.
    pub fn is_panel_visible(&self, panel: PanelId) -> bool {
        self.visible && self.panel_enabled(panel)
    }

    fn panel_enabled(&self, panel: PanelId) -> bool {
        let index = match panel {
            PanelId::Performance => 0,
            PanelId::Ecs => 1,
            PanelId::Console => 2,
        };

        self.panels[index]
    }

    fn set_panel_enabled(&mut self, panel: PanelId, enabled: bool) {
        let index = match panel {
            PanelId::Performance => 0,
            PanelId::Ecs => 1,
            PanelId::Console => 2,
        };

        self.panels[index] = enabled;
    }

    /// Whether `entity`'s component list is expanded.
    pub fn is_expanded(&self, entity: Entity) -> bool {
        self.expanded.contains(&entity)
    }

    /// Whether the toggle key was down on the previous frame.
    pub fn toggle_was_pressed(&self) -> bool {
        self.toggle_was_pressed
    }

    /// Records the toggle key's state for the next frame's edge detection.
    pub fn set_toggle_was_pressed(&mut self, pressed: bool) {
        self.toggle_was_pressed = pressed;
    }

    /// Flips the overlay's visibility, logging the new state.
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;

        // Turning the overlay back on is also the recovery path for panels the
        // user closed with their X buttons.
        if self.visible {
            self.panels = [true; 3];
        }

        self.dirty = true;
    }

    /// Applies a message to this state.
    ///
    /// Kept separate from the view so the panels stay pure functions of the
    /// ECS, which is the model `orbital_iced` uses everywhere else.
    pub fn apply(&mut self, message: &DebugMessage) {
        match message {
            DebugMessage::SetTab(tab) => {
                self.active_tab = *tab;
                self.dirty = true;
            }
            DebugMessage::SelectEntity(entity) => {
                // Moving the selection collapses the entity that was selected
                // before, since its fields are no longer shown. Clicking the
                // selected row again sends `None`, which also collapses it.
                if self.selected != *entity
                    && let Some(previous) = self.selected
                {
                    self.expanded.remove(&previous);
                    self.last_error = None;
                }

                self.selected = *entity;
                // Expand on select so the components are immediately visible.
                if let Some(entity) = *entity {
                    self.expanded.insert(entity);
                }
                self.dirty = true;
            }
            DebugMessage::ToggleEntity(entity) => {
                if !self.expanded.remove(entity) {
                    self.expanded.insert(*entity);
                }
                self.dirty = true;
            }
            DebugMessage::SetFilter(filter) => {
                self.filter = filter.clone();
                self.dirty = true;
            }
            DebugMessage::ToggleGraphSeries(series) => {
                let valid = match series {
                    GraphSeries::Stage(stage) => *stage < TIMING_STAGES.len(),
                    GraphSeries::Fps | GraphSeries::Cdt => true,
                };

                if valid {
                    if !self.graph_series.remove(series) {
                        self.graph_series.insert(*series);
                    }
                    self.dirty = true;
                }
            }
            DebugMessage::SetGraphMetric(metric) => {
                self.graph_metric = *metric;
                self.dirty = true;
            }
            DebugMessage::SetGraphWindow(seconds) => {
                self.set_graph_window(*seconds);
                self.dirty = true;
            }
            DebugMessage::EditGraphWindow(text) => {
                self.graph_window_input = text.clone();
                self.graph_window_error = None;
                self.dirty = true;
            }
            DebugMessage::ApplyGraphWindow => match self.graph_window_input.trim().parse::<f32>() {
                Ok(seconds)
                    if seconds.is_finite()
                        && (MIN_GRAPH_WINDOW_SECS..=MAX_GRAPH_WINDOW_SECS).contains(&seconds) =>
                {
                    self.set_graph_window(seconds);
                }
                Ok(_) => {
                    self.graph_window_error = Some(format!(
                        "window must be between {MIN_GRAPH_WINDOW_SECS} and {MAX_GRAPH_WINDOW_SECS}s"
                    ));
                    self.dirty = true;
                }
                Err(_) => {
                    self.graph_window_error = Some("window must be a number".to_string());
                    self.dirty = true;
                }
            },
            DebugMessage::SetMinLevel(level) => {
                self.min_level = *level;
                self.dirty = true;
            }
            DebugMessage::ClearLog => {
                self.dirty = true;
            }
            DebugMessage::EditField(key, text) => {
                self.field_buffers.insert(key.clone(), text.clone());
                self.dirty = true;
            }
            DebugMessage::ApplyField(_) => {
                // The apply itself happens in `IcedState::handle_message`, which
                // is the only place with both the world and the panel state.
                self.dirty = true;
            }
            DebugMessage::AddModelInstance(_) | DebugMessage::RemoveModelInstance { .. } => {
                // Applied in `handle_debug_message`, which has the world.
                self.dirty = true;
            }
            DebugMessage::TogglePanel(panel) => {
                let enabled = self.panel_enabled(*panel);
                self.set_panel_enabled(*panel, !enabled);
                self.dirty = true;
            }
            DebugMessage::HidePanel(panel) => {
                self.set_panel_enabled(*panel, false);
                self.dirty = true;
            }
        }
    }

    /// Truncates a component's rendered value to [`VALUE_PREVIEW_CHARS`],
    /// marking the cut with an ellipsis.
    pub fn preview(value: &str) -> String {
        if value.chars().count() <= VALUE_PREVIEW_CHARS {
            return value.to_string();
        }

        let mut preview = value.chars().take(VALUE_PREVIEW_CHARS).collect::<String>();
        preview.push('…');
        preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panels_start_visible_but_the_overlay_starts_hidden() {
        let state = DebugUiState::new(winit::keyboard::KeyCode::F1);

        assert!(!state.visible);
        assert!(!state.is_panel_visible(PanelId::Performance));
        assert!(state.panels[1], "ECS panel enabled");
    }

    #[test]
    fn showing_the_overlay_reveals_enabled_panels() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        state.visible = true;

        assert!(state.is_panel_visible(PanelId::Performance));

        // A panel the user dismissed stays hidden even once re-shown.
        state.apply(&DebugMessage::TogglePanel(PanelId::Performance));
        assert!(!state.is_panel_visible(PanelId::Performance));

        state.visible = true;
        assert!(!state.is_panel_visible(PanelId::Performance));
    }

    #[test]
    fn selecting_an_entity_expands_it() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        let entity = Entity::new(7, 0);

        assert!(!state.is_expanded(entity));
        state.apply(&DebugMessage::SelectEntity(Some(entity)));
        assert!(state.is_expanded(entity));
        assert_eq!(state.selected, Some(entity));

        state.apply(&DebugMessage::ToggleEntity(entity));
        assert!(!state.is_expanded(entity));

        // The ECS panel sends `None` when the selected row is clicked again.
        state.apply(&DebugMessage::SelectEntity(None));
        assert_eq!(state.selected, None);
        assert!(
            !state.is_expanded(entity),
            "deselecting collapses the entry it expanded"
        );
    }

    #[test]
    fn moving_the_selection_collapses_the_previous_entity() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        let first = Entity::new(1, 0);
        let second = Entity::new(2, 0);

        state.apply(&DebugMessage::SelectEntity(Some(first)));
        state.last_error = Some("stale error".to_string());

        state.apply(&DebugMessage::SelectEntity(Some(second)));

        assert_eq!(state.selected, Some(second));
        assert!(!state.is_expanded(first), "the old selection collapses");
        assert!(state.is_expanded(second), "the new selection expands");
        assert_eq!(state.last_error, None, "errors belong to the old entity");
    }

    #[test]
    fn graph_series_toggle_and_ignore_out_of_range_stages() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        assert!(
            state
                .graph_series
                .contains(&GraphSeries::Stage(TOTAL_STAGE_INDEX))
        );

        state.apply(&DebugMessage::ToggleGraphSeries(GraphSeries::Stage(2)));
        assert!(state.graph_series.contains(&GraphSeries::Stage(2)));

        state.apply(&DebugMessage::ToggleGraphSeries(GraphSeries::Stage(2)));
        assert!(!state.graph_series.contains(&GraphSeries::Stage(2)));

        state.apply(&DebugMessage::ToggleGraphSeries(GraphSeries::Fps));
        assert!(state.graph_series.contains(&GraphSeries::Fps));

        state.apply(&DebugMessage::ToggleGraphSeries(GraphSeries::Stage(
            TIMING_STAGES.len(),
        )));
        assert!(
            !state
                .graph_series
                .contains(&GraphSeries::Stage(TIMING_STAGES.len())),
            "out-of-range stages are ignored"
        );
    }

    #[test]
    fn graph_window_accepts_a_valid_number_and_rejects_bad_input() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        assert_eq!(state.graph_window_secs, DEFAULT_GRAPH_WINDOW_SECS);

        state.apply(&DebugMessage::EditGraphWindow("25.5".to_string()));
        state.apply(&DebugMessage::ApplyGraphWindow);
        assert_eq!(state.graph_window_secs, 25.5);
        assert!(state.graph_window_error.is_none());

        state.apply(&DebugMessage::EditGraphWindow("100".to_string()));
        state.apply(&DebugMessage::ApplyGraphWindow);
        assert_eq!(state.graph_window_secs, 25.5, "last valid value stays");
        assert!(state.graph_window_error.is_some());

        state.apply(&DebugMessage::EditGraphWindow("nope".to_string()));
        state.apply(&DebugMessage::ApplyGraphWindow);
        assert!(state.graph_window_error.is_some());
    }

    #[test]
    fn slider_window_values_are_clamped() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);

        state.apply(&DebugMessage::SetGraphWindow(0.1));
        assert_eq!(state.graph_window_secs, MIN_GRAPH_WINDOW_SECS);

        state.apply(&DebugMessage::SetGraphWindow(1000.0));
        assert_eq!(state.graph_window_secs, MAX_GRAPH_WINDOW_SECS);
    }

    #[test]
    fn graph_metric_defaults_to_last_and_can_change() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        assert_eq!(state.graph_metric, GraphMetric::Last);

        state.apply(&DebugMessage::SetGraphMetric(GraphMetric::All));
        assert_eq!(state.graph_metric, GraphMetric::All);
    }

    #[test]
    fn hiding_a_panel_only_hides_that_panel() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        state.visible = true;
        state.apply(&DebugMessage::HidePanel(PanelId::Ecs));

        assert!(state.visible, "the overlay itself stays open");
        assert!(!state.is_panel_visible(PanelId::Ecs));
        assert!(state.is_panel_visible(PanelId::Performance));
    }

    #[test]
    fn showing_the_overlay_again_restores_closed_panels() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        state.apply(&DebugMessage::HidePanel(PanelId::Console));

        state.toggle_visibility();
        assert!(state.is_panel_visible(PanelId::Console));
    }

    #[test]
    fn preview_truncates_on_a_char_boundary() {
        // Multi-byte characters, so a byte-based cut would land mid-codepoint.
        let preview = DebugUiState::preview(&"αβγδε".repeat(100));
        assert_eq!(preview.chars().count(), VALUE_PREVIEW_CHARS + 1);
        assert!(preview.ends_with('…'));

        // A value that fits is returned untouched.
        assert_eq!(DebugUiState::preview("short"), "short");
    }
}
