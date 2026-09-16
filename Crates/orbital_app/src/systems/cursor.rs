use orbital_ecs_bridge::{EngineEvent, EngineEvents, InputSnapshot};
use orbital_input::{InputButton, InputState};
use winit::keyboard::{KeyCode, PhysicalKey};

use orbital_ecs::{Commands, ComponentAccess, System, World};

/// Toggles cursor grab on Escape press.
///
/// Tracks the previous frame's Escape state for edge detection so the toggle
/// only fires once per press. Pushes [`EngineEvent::CursorGrabbed`] and
/// [`EngineEvent::CursorVisible`] events; the runtime processes them after
/// schedules complete.
pub struct CursorToggle {
    grabbed: bool,
    was_pressed: bool,
    access: ComponentAccess,
}

impl CursorToggle {
    pub fn new(initially_grabbed: bool) -> Self {
        Self {
            grabbed: initially_grabbed,
            was_pressed: false,
            access: ComponentAccess::new(),
        }
    }
}

impl System for CursorToggle {
    fn name(&self) -> &str {
        "cursor_toggle"
    }

    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, _commands: &mut Commands) {
        let input = match world.get_resource::<InputSnapshot>() {
            Some(i) => i,
            None => return,
        };

        let escape_pressed = is_escape_pressed(&input.0);

        // Edge detection: toggle only on fresh press (not held)
        if escape_pressed && !self.was_pressed {
            self.grabbed = !self.grabbed;
            if let Some(mut events) = world.get_resource_mut::<EngineEvents>() {
                events.0.push(EngineEvent::CursorGrabbed(self.grabbed));
                events.0.push(EngineEvent::CursorVisible(self.grabbed));
            }
        }
        self.was_pressed = escape_pressed;
    }
}

fn is_escape_pressed(input: &InputState) -> bool {
    input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::Escape)))
        .map(|(_, pressed)| pressed)
        .unwrap_or(false)
}
