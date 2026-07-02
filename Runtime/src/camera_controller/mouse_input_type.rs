use crate::app::input::{InputButton, InputState};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CameraControllerMouseInputType {
    /// Rotate the camera via the mouse always.
    Always,
    /// Rotate the camera via the mouse only if any of these buttons are pressed.
    OnlyWithButton { buttons: Vec<InputButton> },
}

impl CameraControllerMouseInputType {
    pub fn is_triggering(&self, input_state: &InputState) -> bool {
        match self {
            CameraControllerMouseInputType::Always => true,
            CameraControllerMouseInputType::OnlyWithButton { buttons } => {
                buttons.iter().any(|button| {
                    if let Some((_, pressed)) = input_state.button_state_any(button) {
                        pressed
                    } else {
                        false
                    }
                })
            }
        }
    }
}
