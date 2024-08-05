use std::f32::consts::PI;

use cgmath::Point3;
use gilrs::Axis;
use hashbrown::HashMap;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::util::{Action, InputBinding};

pub struct CameraSettings {
    /// Identifier of the Camera
    pub identifier: &'static str,

    /// Add desired [InputBinding]s here.
    ///
    /// The [HashMap] is expecting a [InputBinding] => [Action]
    /// mapping.
    /// You **must** use _predefined_ [Action]s for this to work.
    /// Check __**const**__ variables of [CameraSettings], like:
    /// [CameraSettings::ACTION_BINARY_MOVE_FORWARD].
    ///
    /// Not using a given [Action] simply disables the feature.
    pub input_bindings: HashMap<InputBinding, Action>,

    /// Defines if the Camera should react to mouse movements.  
    /// If `true`, moves with the mouse.  
    /// If `false`, does not move with the mouse.
    ///
    /// This will also enable mouse cursor snapping and hiding.
    /// Meaning: After each cycle, the mouse cursor will be
    /// returned to it's origin point (= center of the window)
    /// to be used **after input events** for determining how much
    /// the cursor deviated from it's origin.
    pub move_camera_with_mouse: bool,

    /// Defines the starting position of the Camera.
    pub start_position: Option<Point3<f32>>,

    /// Defines the starting pitch angle of the Camera.
    pub start_pitch: Option<f32>,

    /// Defines the starting yaw angle of the Camera.
    pub start_yaw: Option<f32>,

    /// Defines a modifier that is multiplied with the given input vectors upon
    /// movement. This value is applied no matter which input source it comes
    /// from.  
    /// `1.0` will default to the raw input vector.
    /// `< 1.0 && > 0.0` will decrease the speed.
    /// `> 1.0` will increase the speed.
    pub movement_speed: f32,

    /// Defines a modifier that is multiplied with the given input vector upon
    /// changing yaw/pitch values. This value is only applied to gamepad input
    /// sources.  
    /// `1.0` will default to the raw input vector.
    /// `< 1.0 && > 0.0` will decrease the sensitivity.
    /// `> 1.0` will increase the sensitivity.
    pub looking_gamepad_sensitivity: f32,

    /// Defines a modifier that is multiplied with the given input vector upon
    /// changing yaw/pitch values. This value is only applied to mouse input
    /// sources.  
    /// `1.0` will default to the raw input vector.
    /// `< 1.0 && > 0.0` will decrease the sensitivity.
    /// `> 1.0` will increase the sensitivity.
    pub looking_mouse_sensitivity: f32,
}

impl CameraSettings {
    // Binary movement action names
    pub const ACTION_BINARY_MOVE_FORWARD: Action = "move_forward";
    pub const ACTION_BINARY_MOVE_BACKWARD: Action = "move_backward";
    pub const ACTION_BINARY_MOVE_LEFT: Action = "move_left";
    pub const ACTION_BINARY_MOVE_RIGHT: Action = "move_right";
    pub const ACTION_BINARY_MOVE_DOWN: Action = "move_down";
    pub const ACTION_BINARY_MOVE_UP: Action = "move_up";

    // Binary looking action names
    pub const ACTION_BINARY_LOOK_LEFT: Action = "look_left";
    pub const ACTION_BINARY_LOOK_RIGHT: Action = "look_right";
    pub const ACTION_BINARY_LOOK_DOWN: Action = "look_down";
    pub const ACTION_BINARY_LOOK_UP: Action = "look_up";

    // Variable movement action names
    pub const ACTION_VARIABLE_MOVE_FORWARD_BACKWARD: Action = "move_forward_backward";
    pub const ACTION_VARIABLE_MOVE_LEFT_RIGHT: Action = "move_left_right";
    pub const ACTION_VARIABLE_MOVE_UP_DOWN: Action = "move_up_down";

    // Variable looking action names
    pub const ACTION_VARIABLE_LOOK_LEFT_RIGHT: Action = "look_left_right";
    pub const ACTION_VARIABLE_LOOK_UP_DOWN: Action = "look_up_down";
}

impl Default for CameraSettings {
    fn default() -> Self {
        let mut input_bindings = HashMap::new();

        input_bindings.insert(
            InputBinding::KeyboardKey(PhysicalKey::Code(KeyCode::KeyW)),
            Self::ACTION_BINARY_MOVE_FORWARD,
        );
        input_bindings.insert(
            InputBinding::KeyboardKey(PhysicalKey::Code(KeyCode::KeyS)),
            Self::ACTION_BINARY_MOVE_BACKWARD,
        );
        input_bindings.insert(
            InputBinding::KeyboardKey(PhysicalKey::Code(KeyCode::KeyA)),
            Self::ACTION_BINARY_MOVE_LEFT,
        );
        input_bindings.insert(
            InputBinding::KeyboardKey(PhysicalKey::Code(KeyCode::KeyD)),
            Self::ACTION_BINARY_MOVE_RIGHT,
        );
        input_bindings.insert(
            InputBinding::KeyboardKey(PhysicalKey::Code(KeyCode::KeyQ)),
            Self::ACTION_BINARY_MOVE_DOWN,
        );
        input_bindings.insert(
            InputBinding::KeyboardKey(PhysicalKey::Code(KeyCode::KeyE)),
            Self::ACTION_BINARY_MOVE_UP,
        );
        input_bindings.insert(
            InputBinding::GamepadAxis(Axis::LeftStickX),
            Self::ACTION_VARIABLE_MOVE_FORWARD_BACKWARD,
        );
        input_bindings.insert(
            InputBinding::GamepadAxis(Axis::LeftStickY),
            Self::ACTION_VARIABLE_MOVE_LEFT_RIGHT,
        );
        input_bindings.insert(
            InputBinding::GamepadAxis(Axis::RightStickY),
            Self::ACTION_VARIABLE_LOOK_LEFT_RIGHT,
        );
        input_bindings.insert(
            InputBinding::GamepadAxis(Axis::RightStickX),
            Self::ACTION_VARIABLE_LOOK_UP_DOWN,
        );

        Self {
            identifier: "Default",
            input_bindings,
            move_camera_with_mouse: true,
            start_position: Some(Point3::new(5.0, 0.0, 0.0)),
            start_pitch: Some(0.0),
            start_yaw: Some(PI),
            movement_speed: 5.0,
            looking_gamepad_sensitivity: 2.5,
            looking_mouse_sensitivity: 2.5,
        }
    }
}
