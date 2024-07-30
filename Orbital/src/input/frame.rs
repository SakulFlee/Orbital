use hashbrown::{HashMap, HashSet};
use winit::dpi::PhysicalPosition;

use super::{
    action::Action,
    layout::{Layout, GLOBAL_LAYOUT},
};

#[derive(Debug, Default)]
pub struct InputFrame {
    pub actions: HashMap<Layout, HashSet<Action>>,
    pub axis: HashMap<Layout, HashMap<Action, f64>>,
    pub cursor_position: PhysicalPosition<f64>,
    pub cursor_inside_window: bool,
    pub mouse_scroll: (f32, f32),
}

impl InputFrame {
    pub fn new() -> Self {
        InputFrame::default()
    }

    pub fn is_active(&self, action: &Action) -> bool {
        self.is_active_group(action, &GLOBAL_LAYOUT)
    }

    pub fn is_active_group(&self, action: &Action, group: &Layout) -> bool {
        if let Some(actions) = self.actions.get(group) {
            return actions.contains(action);
        }

        return false;
    }

    // TODO: Function to check if an axis corresponds to an action and if, return the value, otherwise check if it's a binary action and return 0.0/1.0f
}
