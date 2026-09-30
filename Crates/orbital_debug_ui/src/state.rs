//! Shared UI state and message type for the debug panels.

use std::collections::HashSet;

use orbital_ecs::Entity;

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
    /// Show only entities whose index or component names match.
    SetFilter(String),
    /// Show only log records at or above this level.
    SetMinLevel(log::LevelFilter),
    /// Drop the captured log buffer.
    ClearLog,
    /// Flip a panel's visibility.
    TogglePanel(PanelId),
    /// Close a panel's window.
    ClosePanel,
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
    /// Entity-tree filter; matches the entity index or a component type name.
    pub filter: String,
    /// Log-level filter for the console.
    pub min_level: log::LevelFilter,
    /// Set whenever something changed that the panels should redraw for.
    ///
    /// Not yet used for gating the rebuild — see
    /// [`Self::refresh_interval`] — but kept as the single place a future
    /// dirty-flag implementation can hook into.
    pub dirty: bool,
    /// How long a panel may reuse its previous view before rebuilding.
    pub refresh_interval: std::time::Duration,
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
            min_level: log::LevelFilter::Info,
            dirty: true,
            // Fast enough that a value changed by a click shows up immediately,
            // slow enough that holding a slider or typing in the filter does not
            // rebuild every frame.
            refresh_interval: std::time::Duration::from_millis(50),
            toggle_key,
            toggle_was_pressed: false,
        }
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
            DebugMessage::SetMinLevel(level) => {
                self.min_level = *level;
                self.dirty = true;
            }
            DebugMessage::ClearLog => {
                self.dirty = true;
            }
            DebugMessage::TogglePanel(panel) => {
                let enabled = self.panel_enabled(*panel);
                self.set_panel_enabled(*panel, !enabled);
                self.dirty = true;
            }
            DebugMessage::ClosePanel => {
                self.visible = false;
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
    }

    #[test]
    fn close_hides_everything() {
        let mut state = DebugUiState::new(winit::keyboard::KeyCode::F1);
        state.visible = true;
        state.apply(&DebugMessage::ClosePanel);
        assert!(!state.visible);
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
