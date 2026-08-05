use orbital_ecs::{Res, ResMut};
use orbital_ecs_bridge::InputSnapshot;
use orbital_input::InputButton;
use winit::keyboard::PhysicalKey;

use crate::state::EguiState;

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
