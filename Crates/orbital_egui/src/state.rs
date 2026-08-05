use egui::Context;
use winit::keyboard::KeyCode;

use crate::ui::Panel;

/// Core egui state stored as an ECS resource.
///
/// Contains the shared [`Context`], visibility flag, and toggle keybinding.
/// Systems read [`EguiState::enabled`] to check if the UI is visible.
pub struct EguiState {
    pub ctx: Context,
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

/// ECS resource holding all registered egui panels.
///
/// Any module can add panels by inserting/updating this resource:
///
/// ```ignore
/// if let Some(mut panels) = ecs.get_resource_mut::<EguiPanels>() {
///     panels.0.push(Box::new(MyCustomPanel));
/// }
/// ```
///
/// The egui overlay reads this resource each frame and calls [`Panel::ui`]
/// for every registered panel.
pub struct EguiPanels(pub Vec<Box<dyn Panel>>);
