use crate::app::input::{InputAxis, InputButton};
use crate::camera_controller::{
    CameraControllerAxisInputMode, CameraControllerButtonInputMode, CameraControllerMouseInputMode,
    CameraControllerMouseInputType,
};
use winit::event::MouseButton;

#[derive(Debug, Clone, PartialEq)]
pub enum CameraControllerRotationType {
    /// A free camera controller that can move around freely without any constraints.
    Free {
        /// Controls mouse behavior
        mouse_input: Option<CameraControllerMouseInputMode>,
        /// Controls which axis can rotate the camera.
        axis_input: Option<CameraControllerAxisInputMode>,
        /// Controls which buttons can move the camera.
        button_input: Option<CameraControllerButtonInputMode>,
        /// If true, the camera will ignore the pitch when moving forward.
        /// Meaning, only the yaw value will be used to determine where "forward" is.
        /// If false, the camera will take pitch into consideration.
        ///
        /// In most cases you want this set to true so that the forward vector doesn't move the
        /// camera up and down.
        /// However, there are some exceptions like, for example, _"creative flight"_, _free flight_,
        /// diving/swimming or space.
        /// TODO: Check if it's actually pitch and not yaw
        ignore_pitch_for_forward_movement: bool,
    },
    /// A camera controller that is locked and will not be rotated automatically.
    /// A locked camera can still be interacted with and manually rotated!
    Locked,
}

impl CameraControllerRotationType {
    pub fn default_3rd_person() -> Self {
        Self::Free {
            mouse_input: Some(CameraControllerMouseInputMode {
                input_type: CameraControllerMouseInputType::Always,
                sensitivity: 2.0,
                grab_cursor: true,
                hide_cursor: true,
            }),
            axis_input: Some(CameraControllerAxisInputMode {
                input_type: vec![InputAxis::GamepadRightStick],
                sensitivity: 2.5,
            }),
            button_input: None,
            ignore_pitch_for_forward_movement: true,
        }
    }

    pub fn default_1st_person() -> Self {
        Self::Free {
            mouse_input: Some(CameraControllerMouseInputMode {
                input_type: CameraControllerMouseInputType::Always,
                sensitivity: 2.0,
                grab_cursor: true,
                hide_cursor: true,
            }),
            axis_input: Some(CameraControllerAxisInputMode {
                input_type: vec![InputAxis::GamepadRightStick],
                sensitivity: 2.5,
            }),
            button_input: None,
            ignore_pitch_for_forward_movement: true,
        }
    }

    pub fn default_free_flight() -> Self {
        Self::Free {
            mouse_input: Some(CameraControllerMouseInputMode {
                input_type: CameraControllerMouseInputType::Always,
                sensitivity: 2.0,
                grab_cursor: true,
                hide_cursor: true,
            }),
            axis_input: Some(CameraControllerAxisInputMode {
                input_type: vec![InputAxis::GamepadRightStick],
                sensitivity: 2.5,
            }),
            button_input: None,
            ignore_pitch_for_forward_movement: false,
        }
    }

    pub fn default_top_down() -> Self {
        Self::Free {
            mouse_input: Some(CameraControllerMouseInputMode {
                input_type: CameraControllerMouseInputType::OnlyWithButton {
                    buttons: vec![InputButton::Mouse(MouseButton::Middle)],
                },
                sensitivity: 2.0,
                grab_cursor: true,
                hide_cursor: true,
            }),
            axis_input: Some(CameraControllerAxisInputMode {
                input_type: vec![InputAxis::GamepadRightStick],
                sensitivity: 2.5,
            }),
            button_input: None,
            ignore_pitch_for_forward_movement: true,
        }
    }
}
