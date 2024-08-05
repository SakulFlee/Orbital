use log::debug;
use ulid::Ulid;

use crate::{
    app::{AppChange, InputEvent},
    game::{Element, ElementRegistration, WorldChange},
    resources::descriptors::CameraDescriptor,
    util::{Action, InputBinding, InputHandler},
};

use super::settings::CameraSettings;

pub struct StandardCamera {
    settings: CameraSettings,
    input_handler: InputHandler,

    // --- Flags for input handling ---
    has_action_binary_move_forward: bool,
    has_action_binary_move_backward: bool,
    has_action_binary_move_left: bool,
    has_action_binary_move_right: bool,
    has_action_binary_move_down: bool,
    has_action_binary_move_up: bool,
    has_action_binary_look_left: bool,
    has_action_binary_look_right: bool,
    has_action_binary_look_down: bool,
    has_action_binary_look_up: bool,
    has_action_variable_move_forward_backward: bool,
    has_action_variable_move_left_right: bool,
    has_action_variable_move_up_down: bool,
    has_action_variable_look_left_right: bool,
    has_action_variable_look_up_down: bool,
}

impl StandardCamera {
    pub const DEFAULT_TAG: &'static str = "StandardCamera";

    pub fn new(settings: CameraSettings) -> Self {
        let mut s = Self {
            settings,
            input_handler: InputHandler::new(),
            has_action_binary_move_forward: false,
            has_action_binary_move_backward: false,
            has_action_binary_move_left: false,
            has_action_binary_move_right: false,
            has_action_binary_move_down: false,
            has_action_binary_move_up: false,
            has_action_binary_look_left: false,
            has_action_binary_look_right: false,
            has_action_binary_look_down: false,
            has_action_binary_look_up: false,
            has_action_variable_move_forward_backward: false,
            has_action_variable_move_left_right: false,
            has_action_variable_move_up_down: false,
            has_action_variable_look_left_right: false,
            has_action_variable_look_up_down: false,
        };

        s.register_input_mappings();

        s
    }

    fn register_input_mappings(&mut self) -> InputHandler {
        let mut input_handler = InputHandler::new();

        self.settings
            .input_bindings
            .iter()
            .for_each(|(binding, action)| {
                match *action {
                    CameraSettings::ACTION_BINARY_MOVE_FORWARD => {
                        self.has_action_binary_move_forward = true
                    }
                    CameraSettings::ACTION_BINARY_MOVE_BACKWARD => {
                        self.has_action_binary_move_backward = true
                    }
                    CameraSettings::ACTION_BINARY_MOVE_LEFT => {
                        self.has_action_binary_move_left = true
                    }
                    CameraSettings::ACTION_BINARY_MOVE_RIGHT => {
                        self.has_action_binary_move_right = true
                    }
                    CameraSettings::ACTION_BINARY_MOVE_DOWN => {
                        self.has_action_binary_move_down = true
                    }
                    CameraSettings::ACTION_BINARY_MOVE_UP => self.has_action_binary_move_up = true,
                    CameraSettings::ACTION_BINARY_LOOK_LEFT => {
                        self.has_action_binary_look_left = true
                    }
                    CameraSettings::ACTION_BINARY_LOOK_RIGHT => {
                        self.has_action_binary_look_right = true
                    }
                    CameraSettings::ACTION_BINARY_LOOK_DOWN => {
                        self.has_action_binary_look_down = true
                    }
                    CameraSettings::ACTION_BINARY_LOOK_UP => self.has_action_binary_look_up = true,
                    CameraSettings::ACTION_VARIABLE_MOVE_FORWARD_BACKWARD => {
                        self.has_action_variable_move_forward_backward = true
                    }
                    CameraSettings::ACTION_VARIABLE_MOVE_LEFT_RIGHT => {
                        self.has_action_variable_move_left_right = true
                    }
                    CameraSettings::ACTION_VARIABLE_MOVE_UP_DOWN => {
                        self.has_action_variable_move_up_down = true
                    }
                    CameraSettings::ACTION_VARIABLE_LOOK_LEFT_RIGHT => {
                        self.has_action_variable_look_left_right = true
                    }
                    CameraSettings::ACTION_VARIABLE_LOOK_UP_DOWN => {
                        self.has_action_variable_look_up_down = true
                    }
                    _ => (),
                }

                match binding {
                    InputBinding::KeyboardKey(key) => {
                        input_handler.register_keyboard_mapping(*key, action)
                    }
                    InputBinding::MouseButton(button) => {
                        input_handler.register_mouse_button_mapping(*button, action)
                    }
                    InputBinding::GamepadButton(button) => {
                        input_handler.register_gamepad_button_mapping(*button, action)
                    }
                    InputBinding::GamepadAxis(axis) => {
                        input_handler.register_gamepad_axis_mapping(*axis, action)
                    }
                }
            });

        input_handler
    }

    fn read_input_vector(
        &self,
        axis: Option<(Action, Action)>,
        button_axis: Option<(Action, Action, Action, Action)>,
        include_cursor: bool,
    ) -> Option<(bool, f32, f32)> {
        if let Some((x_axis, y_axis)) = axis {
            if let Some(x_value) = self.input_handler.get_only_axis(x_axis) {
                if let Some(y_value) = self.input_handler.get_only_axis(y_axis) {
                    return Some((true, x_value, y_value));
                }
            }
        }

        if include_cursor {
            let (x, y) = self.input_handler.calculate_view_change_from_cursor();
            return Some((false, x, y));
        }

        if let Some((x_pos, x_neg, y_pos, y_neg)) = button_axis {
            let x = if let Some(value) = self.input_handler.get_button_axis(x_pos, x_neg) {
                value
            } else {
                debug!("X HIT");
                0.0
            };

            let y = if let Some(value) = self.input_handler.get_button_axis(y_pos, y_neg) {
                value
            } else {
                debug!("Y HIT");
                0.0
            };

            return Some((false, x, y));
        }

        None
    }

    fn read_input_vector_singular(
        &self,
        axis: Option<Action>,
        button_axis: Option<(Action, Action)>,
    ) -> Option<(bool, f32)> {
        if let Some(axis) = axis {
            if let Some(value) = self.input_handler.get_only_axis(axis) {
                return Some((true, value));
            }
        }

        if let Some((pos, neg)) = button_axis {
            let value = self
                .input_handler
                .get_button_axis(pos, neg)
                .or(Some(0.0))
                .unwrap();

            return Some((false, value));
        }

        None
    }
}

impl Element for StandardCamera {
    fn on_registration(&mut self, _ulid: &Ulid) -> ElementRegistration {
        let mut descriptor = CameraDescriptor {
            identifier: self.settings.identifier.into(),
            ..Default::default()
        };

        if let Some(x) = self.settings.start_position {
            descriptor.position = x;
        }

        if let Some(x) = self.settings.start_pitch {
            descriptor.pitch = x;
        }

        if let Some(x) = self.settings.start_yaw {
            descriptor.yaw = x;
        }

        let mut world_changes = vec![WorldChange::SpawnCameraAndMakeActive(descriptor)];

        if self.settings.move_camera_with_mouse {
            world_changes.push(WorldChange::AppChange(AppChange::ChangeCursorVisible(
                false,
            )));
        }

        ElementRegistration {
            tags: Some(vec![
                Self::DEFAULT_TAG.into(),
                self.settings.identifier.into(),
            ]),
            world_changes: Some(world_changes),
            ..Default::default()
        }
    }

    fn on_input_event(&mut self, _delta_time: f64, input_event: &InputEvent) {
        self.input_handler.handle_event(input_event);
    }

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        debug!(
            "STATE +: {}",
            self.input_handler
                .is_triggered(CameraSettings::ACTION_BINARY_MOVE_FORWARD)
        );
        debug!(
            "STATE -: {}",
            self.input_handler
                .is_triggered(CameraSettings::ACTION_BINARY_MOVE_BACKWARD)
        );

        let move_forward_backward_left_right = if (self.has_action_variable_move_forward_backward
            && self.has_action_variable_move_left_right)
            || (self.has_action_binary_move_forward
                && self.has_action_binary_move_backward
                && self.has_action_binary_look_left
                && self.has_action_binary_move_right)
        {
            self.read_input_vector(
                Some((
                    CameraSettings::ACTION_VARIABLE_MOVE_FORWARD_BACKWARD,
                    CameraSettings::ACTION_VARIABLE_LOOK_LEFT_RIGHT,
                )),
                Some((
                    CameraSettings::ACTION_BINARY_MOVE_FORWARD,
                    CameraSettings::ACTION_BINARY_MOVE_BACKWARD,
                    CameraSettings::ACTION_BINARY_MOVE_LEFT,
                    CameraSettings::ACTION_BINARY_MOVE_RIGHT,
                )),
                false,
            )
        } else if self.has_action_variable_move_forward_backward
            && self.has_action_binary_move_forward
            && self.has_action_binary_move_backward
        {
            let x = self.read_input_vector_singular(
                Some(CameraSettings::ACTION_VARIABLE_MOVE_FORWARD_BACKWARD),
                Some((
                    CameraSettings::ACTION_BINARY_MOVE_FORWARD,
                    CameraSettings::ACTION_BINARY_MOVE_BACKWARD,
                )),
            );

            if let Some((is_axis, axis_value)) = x {
                Some((is_axis, axis_value, 0.0))
            } else {
                None
            }
        } else if self.has_action_variable_move_left_right
            && self.has_action_binary_move_left
            && self.has_action_binary_move_right
        {
            let y = self.read_input_vector_singular(
                Some(CameraSettings::ACTION_VARIABLE_MOVE_LEFT_RIGHT),
                Some((
                    CameraSettings::ACTION_BINARY_MOVE_LEFT,
                    CameraSettings::ACTION_BINARY_MOVE_RIGHT,
                )),
            );

            if let Some((is_axis, axis_value)) = y {
                Some((is_axis, 0.0, axis_value))
            } else {
                None
            }
        } else {
            None
        };

        let look_up_down_left_right = if (self.has_action_variable_look_up_down
            && self.has_action_variable_look_left_right)
            || (self.has_action_binary_look_up
                && self.has_action_binary_look_down
                && self.has_action_binary_look_left
                && self.has_action_binary_look_right)
        {
            self.read_input_vector(
                Some((
                    CameraSettings::ACTION_VARIABLE_LOOK_UP_DOWN,
                    CameraSettings::ACTION_VARIABLE_LOOK_LEFT_RIGHT,
                )),
                Some((
                    CameraSettings::ACTION_BINARY_LOOK_UP,
                    CameraSettings::ACTION_BINARY_LOOK_DOWN,
                    CameraSettings::ACTION_BINARY_LOOK_LEFT,
                    CameraSettings::ACTION_BINARY_LOOK_RIGHT,
                )),
                true,
            )
        } else if self.has_action_variable_look_up_down
            && self.has_action_binary_look_up
            && self.has_action_binary_look_down
        {
            let x = self.read_input_vector_singular(
                Some(CameraSettings::ACTION_VARIABLE_LOOK_UP_DOWN),
                Some((
                    CameraSettings::ACTION_BINARY_LOOK_UP,
                    CameraSettings::ACTION_BINARY_LOOK_DOWN,
                )),
            );

            if let Some((is_axis, axis_value)) = x {
                Some((is_axis, axis_value, 0.0))
            } else {
                let delta = self.input_handler.calculate_view_change_from_cursor();
                Some((false, delta.0, delta.1))
            }
        } else if self.has_action_variable_look_left_right
            && self.has_action_binary_look_left
            && self.has_action_binary_look_right
        {
            let y = self.read_input_vector_singular(
                Some(CameraSettings::ACTION_VARIABLE_LOOK_LEFT_RIGHT),
                Some((
                    CameraSettings::ACTION_BINARY_LOOK_LEFT,
                    CameraSettings::ACTION_BINARY_LOOK_RIGHT,
                )),
            );

            if let Some((is_axis, axis_value)) = y {
                Some((is_axis, 0.0, axis_value))
            } else {
                let delta = self.input_handler.calculate_view_change_from_cursor();
                Some((false, delta.0, delta.1))
            }
        } else {
            None
        };

        debug!(
            "\nMove: {:?}\nLook: {:?}",
            move_forward_backward_left_right, look_up_down_left_right
        );

        // TODO ... use? xD GL LOL

        None
    }
}
