use cgmath::{Vector2, Zero};
use gilrs::Axis;
use hashbrown::HashMap;
use log::debug;
use winit::event::{ElementState, MouseScrollDelta};

use super::{InputAxis, InputButton, InputEvent, InputId};

#[derive(Debug, Clone)]
pub struct InputState {
    button_states: HashMap<InputId, HashMap<InputButton, bool>>,
    delta_states: HashMap<InputId, HashMap<InputAxis, Vector2<f64>>>,
    mouse_cursor_position_state: Vector2<f64>,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            button_states: HashMap::new(),
            delta_states: HashMap::new(),
            mouse_cursor_position_state: Vector2::zero(),
        }
    }

    /// Resets all delta values back to zero.
    /// This should be called after updating, but before the next cycle.
    /// I.e. after rendering is a good time.
    pub fn reset_deltas(&mut self) {
        self.delta_states.iter_mut().for_each(|(_, state)| {
            state
                .iter_mut()
                // Only reset mouse deltas
                .filter(|(axis, _)| {
                    InputAxis::MouseMovement.eq(axis) || InputAxis::MouseScrollWheel.eq(axis)
                })
                .for_each(|(_, delta)| *delta = Vector2::zero())
        });
    }

    /// Handles input events to populate the input state.
    pub fn handle_event(&mut self, input_event: InputEvent) {
        let (input_id, input_button_state, input_axis_state): (
            InputId,
            Option<(InputButton, bool)>,
            Option<(InputAxis, Vector2<f64>)>,
        ) = match input_event {
            InputEvent::KeyboardButton {
                device_id,
                event,
                is_synthetic: _,
            } => (
                InputId::KeyboardOrMouse(device_id),
                Some((
                    InputButton::Keyboard(event.physical_key),
                    event.state == ElementState::Pressed,
                )),
                None,
            ),
            InputEvent::MouseButton {
                device_id,
                state,
                button,
            } => (
                InputId::KeyboardOrMouse(device_id),
                Some((InputButton::Mouse(button), state == ElementState::Pressed)),
                None,
            ),
            InputEvent::MouseWheel {
                device_id,
                delta,
                phase: _,
            } => {
                let vector_delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Vector2::new(x as f64, -y as f64),
                    MouseScrollDelta::PixelDelta(physical_position) => {
                        Vector2::new(physical_position.x, -physical_position.y)
                    }
                };

                (
                    InputId::KeyboardOrMouse(device_id),
                    None,
                    Some((InputAxis::MouseScrollWheel, vector_delta)),
                )
            }
            InputEvent::MouseMovedPosition {
                device_id: _,
                position,
            } => {
                let vector_delta = Vector2::new(position.x, position.y);

                self.mouse_cursor_position_state = vector_delta;
                return; // No further processing required!
            }
            InputEvent::MouseMovedDelta { device_id, delta } => {
                let vector_delta = Vector2::new(delta.0, delta.1);

                (
                    InputId::KeyboardOrMouse(device_id),
                    None,
                    Some((InputAxis::MouseMovement, vector_delta)),
                )
            }
            #[cfg(feature = "gamepad_input")]
            InputEvent::GamepadButton {
                gamepad_id,
                button,
                pressed,
            } => (
                InputId::Gamepad(gamepad_id),
                Some((InputButton::Gamepad(button), pressed)),
                None,
            ),
            #[cfg(feature = "gamepad_input")]
            InputEvent::GamepadAxis {
                gamepad_id,
                axis,
                value,
            } => {
                let (axis, vector) = match axis {
                    Axis::LeftStickX => {
                        (InputAxis::GamepadLeftStick, Vector2::new(value as f64, 0.0))
                    }
                    Axis::LeftStickY => {
                        (InputAxis::GamepadLeftStick, Vector2::new(0.0, value as f64))
                    }
                    Axis::RightStickX => (
                        InputAxis::GamepadRightStick,
                        Vector2::new(value as f64, 0.0),
                    ),
                    Axis::RightStickY => (
                        InputAxis::GamepadRightStick,
                        Vector2::new(0.0, value as f64),
                    ),
                    Axis::LeftZ => (InputAxis::GamepadTrigger, Vector2::new(value as f64, 0.0)),
                    Axis::RightZ => (InputAxis::GamepadTrigger, Vector2::new(0.0, value as f64)),
                    _ => return,
                };

                (InputId::Gamepad(gamepad_id), None, Some((axis, vector)))
            }
            // Nothing to do, so just return out of here :)
            _ => return,
        };

        if let Some((button, pressed)) = input_button_state {
            self.button_states
                .entry(input_id)
                .or_insert(HashMap::new())
                .entry(button)
                .and_modify(|x| *x = pressed)
                .or_insert(pressed);
        } else if let Some((axis, delta)) = input_axis_state {
            // Our delta has to be flipped here, meaning X = Y and Y = X, since the engine, and thus WGPU and such, use a different coordinate system than what we are reading here.
            // Our "up and down" is Y and our "left and right" is X.
            // In Winit + Gil "up and down" is X and "left and right" is Y.
            // Additionally, the mouse wheel delta for "up" is inverted, so we need to invert that as well.
            // Gamepad inputs will also be clamped to not allow cheating.
            let flipped_delta =
                if InputAxis::MouseMovement.eq(&axis) || InputAxis::MouseScrollWheel.eq(&axis) {
                    Vector2::new(-delta.y, delta.x)
                } else {
                    Vector2::new(delta.y.clamp(-1.0, 1.0), delta.x.clamp(-1.0, 1.0))
                };
            self.delta_states
                .entry(input_id)
                .or_insert(HashMap::new())
                .entry(axis)
                .and_modify(|x| match axis {
                    // Mouse inputs need to be summed as they aren't tracking the mouse position directly, but the change in movement.
                    // After a frame is rendered, we need to reset these.
                    InputAxis::MouseMovement | InputAxis::MouseScrollWheel => *x += flipped_delta,
                    // Gamepad values are stated. Meaning a new input event will always have the total value of the input. Thus, we won't need to summarize here.
                    InputAxis::GamepadLeftStick
                    | InputAxis::GamepadRightStick
                    | InputAxis::GamepadTrigger => {
                        let x_valid = flipped_delta.x.abs() > 0.0001;
                        let y_valid = flipped_delta.y.abs() > 0.0001;

                        if x_valid && y_valid {
                            *x = flipped_delta;
                        } else if x_valid {
                            x.x = flipped_delta.x;
                        } else if y_valid {
                            x.y = flipped_delta.y;
                        } else {
                            *x = Vector2::zero();
                        }
                    }
                })
                .or_insert(flipped_delta);
        }
    }

    pub fn mouse_cursor_position_state(&self) -> Vector2<f64> {
        self.mouse_cursor_position_state
    }

    pub fn button_state_specific(
        &self,
        input_button: &InputButton,
        input_id: InputId,
    ) -> Option<bool> {
        self.button_states
            .get(&input_id)
            .and_then(|x| x.get(input_button))
            .cloned()
    }

    pub fn button_state_any(&self, input_button: &InputButton) -> Option<(InputId, bool)> {
        self.button_states
            .iter()
            .find(|(_, state)| state.contains_key(input_button))
            .and_then(|(input_id, state)| {
                state.get(input_button).map(|pressed| (*input_id, *pressed))
            })
    }

    pub fn button_state_many(
        &self,
        input_buttons: &[&InputButton],
    ) -> HashMap<InputButton, (InputId, bool)> {
        self.button_states
            .iter()
            .flat_map(|(input_id, state)| {
                input_buttons.iter().filter_map(|&input_button| {
                    state
                        .get(input_button)
                        .map(|pressed| (*input_button, (*input_id, *pressed)))
                })
            })
            .collect()
    }

    pub fn button_state_all(&self, input_button: &InputButton) -> Vec<(InputId, bool)> {
        self.button_states
            .iter()
            .filter(|(_, state)| state.contains_key(input_button))
            .filter_map(|(input_id, state)| {
                state.get(input_button).map(|pressed| (*input_id, *pressed))
            })
            .collect()
    }

    pub fn delta_state_specific(
        &self,
        input_axis: &InputAxis,
        input_id: InputId,
    ) -> Option<Vector2<f64>> {
        self.delta_states
            .get(&input_id)
            .and_then(|x| x.get(input_axis))
            .cloned()
    }

    pub fn delta_state_any(&self, input_axis: &InputAxis) -> Option<(InputId, Vector2<f64>)> {
        self.delta_states
            .iter()
            .find(|(_, state)| state.contains_key(input_axis))
            .and_then(|(input_id, state)| state.get(input_axis).map(|delta| (*input_id, *delta)))
    }

    pub fn delta_state_all(&self, input_axis: &InputAxis) -> Vec<(InputId, Vector2<f64>)> {
        self.delta_states
            .iter()
            .filter(|(_, state)| state.contains_key(input_axis))
            .filter_map(|(input_id, state)| state.get(input_axis).map(|delta| (*input_id, *delta)))
            .collect()
    }

    pub fn delta_state_many(
        &self,
        input_axises: &[&InputAxis],
    ) -> HashMap<InputAxis, (InputId, Vector2<f64>)> {
        self.delta_states
            .iter()
            .filter_map(|(input_id, state)| {
                input_axises.iter().find_map(|&input_axis| {
                    state
                        .get(input_axis)
                        .map(|pressed| (*input_axis, (*input_id, *pressed)))
                })
            })
            .collect()
    }

    /// Calculates the current movement vectors for this input state.
    /// Will prioritize gamepad inputs over keyboard inputs.
    /// Uses the button state of any keyboard otherwise.
    ///
    /// # Returns
    /// A tuple of two parts:
    /// 1. A boolean indicating whether the movement vector is from a gamepad.
    /// 2. The movement vector to be used for changing the camera.
    ///
    /// > Note: We might want to multiply this by delta time and add a sensitivity factor!
    pub fn movement_vector(
        &self,
        input_axis: Option<&InputAxis>,
        input_button_forward: &InputButton,
        input_button_backward: &InputButton,
        input_button_left: &InputButton,
        input_button_right: &InputButton,
    ) -> (bool, Vector2<f64>) {
        // Prioritize gamepad inputs
        let gamepad_deltas = input_axis.and_then(|axis| self.delta_state_any(axis));
        if let Some((_, delta)) = gamepad_deltas {
            if !delta.is_zero() {
                return (true, delta);
            }
        }

        let mut movement = Vector2::zero();
        let button_state = self.button_state_many(&[
            input_button_forward,
            input_button_backward,
            input_button_left,
            input_button_right,
        ]);
        for (button, (_, pressed)) in button_state.iter() {
            if !pressed {
                continue;
            }

            if button == input_button_forward {
                movement.x += 1.0;
            } else if button == input_button_backward {
                movement.x -= 1.0;
            } else if button == input_button_left {
                movement.y -= 1.0;
            } else if button == input_button_right {
                movement.y += 1.0;
            }
        }

        (false, movement)
    }

    /// Calculates the current view vectors for this input state.
    /// Will prioritize gamepad inputs over mouse inputs.
    /// Uses the delta movement of any mouse otherwise.
    ///
    /// # Returns
    /// A tuple of two parts:
    /// 1. A boolean indicating whether the view vector is from a gamepad.
    /// 2. The view vector to be used for changing the camera.
    ///
    /// > Note: We might want to multiply this by delta time and add a sensitivity factor!
    pub fn view_vector(&self, gamepad_input_axis: Option<&InputAxis>) -> (bool, Vector2<f64>) {
        // Prioritize gamepad inputs
        let gamepad_deltas = gamepad_input_axis.and_then(|axis| self.delta_state_any(axis));
        if let Some((_, delta)) = gamepad_deltas {
            if !delta.is_zero() {
                return (true, delta);
            }
        }

        if let Some((_, delta)) = self.delta_state_any(&InputAxis::MouseMovement) {
            return (false, delta);
        }

        (false, Vector2::zero())
    }
}
