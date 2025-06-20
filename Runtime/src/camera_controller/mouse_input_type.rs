use crate::app::input::InputButton;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CameraControllerMouseInputType {
    /// Rotate the camera via the mouse always.
    Always,
    /// Rotate the camera via the mouse only if any of these buttons are pressed.
    OnlyWithButton { buttons: Vec<InputButton> },
}
