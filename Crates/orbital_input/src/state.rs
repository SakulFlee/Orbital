use cgmath::{InnerSpace, Vector2, Zero};
#[cfg(feature = "gamepad_input")]
use gilrs::Axis;
use hashbrown::HashMap;
use log::warn;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseScrollDelta, TouchPhase};

use crate::{InputAxis, InputButton, InputEvent, InputId};

#[derive(Debug, Clone)]
pub struct InputState {
    button_states: HashMap<InputId, HashMap<InputButton, bool>>,
    delta_states: HashMap<InputId, HashMap<InputAxis, Vector2<f64>>>,
    mouse_cursor_position_state: Vector2<f64>,
    /// Current position (window pixels) of each active finger, keyed by winit touch id.
    touch_positions: HashMap<u64, Vector2<f64>>,
    /// Position (window pixels) where each active finger first touched down.
    /// Used to implement origin-based (absolute) virtual joysticks.
    touch_origins: HashMap<u64, Vector2<f64>>,
    /// Accumulated movement delta (window pixels) of each finger since the last frame reset.
    touch_deltas: HashMap<u64, Vector2<f64>>,
    surface_size: Option<Vector2<u64>>,
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
            touch_positions: HashMap::new(),
            touch_origins: HashMap::new(),
            touch_deltas: HashMap::new(),
            surface_size: None,
        }
    }

    pub fn reset_deltas(&mut self) {
        self.delta_states.iter_mut().for_each(|(_, state)| {
            state
                .iter_mut()
                .filter(|(axis, _)| {
                    InputAxis::MouseMovement.eq(axis) || InputAxis::MouseScrollWheel.eq(axis)
                })
                .for_each(|(_, delta)| *delta = Vector2::zero())
        });
        self.touch_deltas.clear();
    }

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
                return;
            }
            InputEvent::MouseMovedDelta { device_id, delta } => {
                let vector_delta = Vector2::new(delta.0, delta.1);

                (
                    InputId::KeyboardOrMouse(device_id),
                    None,
                    Some((InputAxis::MouseMovement, vector_delta)),
                )
            }
            InputEvent::Touch {
                device_id,
                phase,
                location,
                id,
                force: _,
            } => {
                let position = Vector2::new(location.x, location.y);
                let input_id = InputId::Touch(device_id);
                let input_button = InputButton::Touch(id);

                match phase {
                    TouchPhase::Started => {
                        self.touch_positions.insert(id, position);
                        self.touch_origins.insert(id, position);
                        self.touch_deltas.remove(&id);
                        self.button_states
                            .entry(input_id)
                            .or_default()
                            .insert(input_button, true);
                    }
                    TouchPhase::Moved => {
                        if let Some(&previous) = self.touch_positions.get(&id) {
                            let delta = position - previous;
                            *self.touch_deltas.entry(id).or_insert(Vector2::zero()) += delta;
                        }
                        self.touch_positions.insert(id, position);
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.touch_positions.remove(&id);
                        self.touch_origins.remove(&id);
                        self.touch_deltas.remove(&id);
                        self.button_states
                            .entry(input_id)
                            .or_default()
                            .insert(input_button, false);
                    }
                }
                return;
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
            let flipped_delta = if InputAxis::MouseMovement.eq(&axis) {
                if let Some(surface_size) = self.surface_size {
                    let half_surface_x = surface_size.x as f64 / 2.0;
                    let half_surface_y = surface_size.y as f64 / 2.0;

                    let new_delta_x = -delta.y / half_surface_x;
                    let new_delta_y = delta.x / half_surface_y;

                    Vector2::new(new_delta_x, new_delta_y)
                } else {
                    warn!("No surface size received yet! Won't normalize input deltas.");
                    Vector2::new(-delta.y, delta.x)
                }
            } else if InputAxis::MouseScrollWheel.eq(&axis) {
                Vector2::new(-delta.y, delta.x)
            } else {
                Vector2::new(delta.y.clamp(-1.0, 1.0), delta.x.clamp(-1.0, 1.0))
            };
            self.delta_states
                .entry(input_id)
                .or_insert(HashMap::new())
                .entry(axis)
                .and_modify(|x| match axis {
                    InputAxis::MouseMovement | InputAxis::MouseScrollWheel => *x += flipped_delta,
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

    pub fn touch_position(&self, finger_id: u64) -> Option<Vector2<f64>> {
        self.touch_positions.get(&finger_id).copied()
    }

    pub fn touch_origin(&self, finger_id: u64) -> Option<Vector2<f64>> {
        self.touch_origins.get(&finger_id).copied()
    }

    pub fn touch_delta(&self, finger_id: u64) -> Option<Vector2<f64>> {
        self.touch_deltas.get(&finger_id).copied()
    }

    /// Positions of all currently active fingers, keyed by winit touch id.
    pub fn touch_positions(&self) -> HashMap<u64, Vector2<f64>> {
        self.touch_positions.clone()
    }

    /// Press-down positions of all currently active fingers, keyed by winit touch id.
    pub fn touch_origins(&self) -> HashMap<u64, Vector2<f64>> {
        self.touch_origins.clone()
    }

    /// Whether any finger is currently touching the screen.
    pub fn has_active_touches(&self) -> bool {
        !self.touch_positions.is_empty()
    }

    /// Number of fingers currently touching the screen.
    pub fn active_touch_count(&self) -> usize {
        self.touch_positions.len()
    }

    /// The most recently reported window (surface) size in physical pixels.
    pub fn surface_size(&self) -> Option<Vector2<u64>> {
        self.surface_size
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

    pub fn delta_state_any_normalized(
        &self,
        input_axis: &InputAxis,
        window_size: Vector2<u32>,
    ) -> Option<(InputId, Vector2<f64>)> {
        if let Some((id, delta_state)) = self.delta_state_any(input_axis) {
            let half_width = window_size.x as f64 / 2.0;
            let half_height = window_size.y as f64 / 2.0;

            let delta_normalized_x = delta_state.x / half_width;
            let delta_normalized_y = delta_state.y / half_height;

            return Some((id, Vector2::new(delta_normalized_x, delta_normalized_y)));
        }

        None
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

    pub fn movement_vector(
        &self,
        input_axis: Option<&InputAxis>,
        input_button_forward: &InputButton,
        input_button_backward: &InputButton,
        input_button_left: &InputButton,
        input_button_right: &InputButton,
    ) -> (bool, Vector2<f64>) {
        let gamepad_deltas = input_axis.and_then(|axis| self.delta_state_any(axis));
        if let Some((_, delta)) = gamepad_deltas
            && !delta.is_zero()
        {
            return (true, delta);
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

    pub fn view_vector(&self, gamepad_input_axis: Option<&InputAxis>) -> (bool, Vector2<f64>) {
        let gamepad_deltas = gamepad_input_axis.and_then(|axis| self.delta_state_any(axis));
        if let Some((_, delta)) = gamepad_deltas
            && !delta.is_zero()
        {
            return (true, delta);
        }

        if let Some((_, delta)) = self.delta_state_any(&InputAxis::MouseMovement) {
            return (false, delta);
        }

        (false, Vector2::zero())
    }

    pub fn surface_resize(&mut self, size: PhysicalSize<u32>) {
        self.surface_size = Some(Vector2::new(size.width as u64, size.height as u64));
    }

    /// Resolve the engine's default touch control scheme from the current
    /// touch state (see [`TouchGesture`]).
    ///
    /// The screen is split in half: the **left half** acts as a virtual
    /// joystick for movement (origin = finger down point), the **right half**
    /// as a drag-to-look zone. When multiple fingers are on a half, the
    /// lowest finger id wins so assignment is deterministic.
    pub fn touch_gesture(&self) -> TouchGesture {
        let Some(size) = self.surface_size else {
            return TouchGesture::default();
        };
        let half_width = size.x as f64 / 2.0;

        let mut move_finger: Option<u64> = None;
        let mut look_finger: Option<u64> = None;
        for &id in self.touch_origins.keys() {
            let origin = self.touch_origins.get(&id).copied().unwrap_or(Vector2::zero());
            if origin.x < half_width {
                if move_finger.is_none_or(|current| id < current) {
                    move_finger = Some(id);
                }
            } else if look_finger.is_none_or(|current| id < current) {
                look_finger = Some(id);
            }
        }

        let mut gesture = TouchGesture {
            active: self.has_active_touches(),
            ..TouchGesture::default()
        };

        if let Some(finger) = move_finger {
            let origin = self.touch_origin(finger).unwrap_or(Vector2::zero());
            let position = self.touch_position(finger).unwrap_or(origin);
            gesture.joystick_origin = Some(origin);
            gesture.joystick_position = Some(position);

            let offset = position - origin;
            let length = offset.magnitude();
            let clamped = if length > TOUCH_JOYSTICK_RADIUS {
                offset * (TOUCH_JOYSTICK_RADIUS / length)
            } else {
                offset
            };
            let magnitude = clamped.magnitude();
            let clamped = if magnitude < TOUCH_JOYSTICK_DEADZONE {
                Vector2::zero()
            } else {
                clamped
            };
            // x: screen-right = strafe right; y: screen-down = backwards, so negate.
            gesture.move_vector = Vector2::new(
                clamped.x / TOUCH_JOYSTICK_RADIUS,
                -clamped.y / TOUCH_JOYSTICK_RADIUS,
            );
        }

        if let Some(finger) = look_finger {
            gesture.look_delta = self.touch_delta(finger).unwrap_or(Vector2::zero());
        }

        gesture
    }
}

/// Radius (in window pixels) defining full deflection of the virtual joystick.
pub const TOUCH_JOYSTICK_RADIUS: f64 = 70.0;

/// Dead-zone radius (in window pixels) below which the joystick reports no movement.
pub const TOUCH_JOYSTICK_DEADZONE: f64 = 10.0;

/// Result of resolving the default touch control scheme (see
/// [`InputState::touch_gesture`]). Shared between the touch camera controller
/// and the on-screen joystick overlay so both always agree on the active
/// movement/look fingers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchGesture {
    /// Whether any finger is touching the screen.
    pub active: bool,
    /// Normalized joystick deflection of the movement finger: `x` = strafe
    /// right, `y` = forward (positive when dragging up). Ranges in `[-1.0, 1.0]`.
    pub move_vector: Vector2<f64>,
    /// Per-frame pixel delta of the look finger (x = horizontal, y = vertical,
    /// in screen coordinates).
    pub look_delta: Vector2<f64>,
    /// Window-pixel position where the movement finger touched down.
    pub joystick_origin: Option<Vector2<f64>>,
    /// Window-pixel current position of the movement finger.
    pub joystick_position: Option<Vector2<f64>>,
}

impl Default for TouchGesture {
    fn default() -> Self {
        Self {
            active: false,
            move_vector: Vector2::zero(),
            look_delta: Vector2::zero(),
            joystick_origin: None,
            joystick_position: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalPosition;
    use winit::event::DeviceId;

    fn touch_event(
        phase: TouchPhase,
        location: (f64, f64),
        id: u64,
    ) -> InputEvent {
        InputEvent::Touch {
            device_id: DeviceId::dummy(),
            phase,
            location: PhysicalPosition::new(location.0, location.1),
            id,
            force: Some(winit::event::Force::Normalized(1.0)),
        }
    }

    fn setup_state() -> InputState {
        let mut state = InputState::new();
        state.surface_resize(PhysicalSize::new(1000, 800));
        state
    }

    #[test]
    fn touch_lifecycle() {
        let mut state = setup_state();

        state.handle_event(touch_event(TouchPhase::Started, (200.0, 300.0), 1));
        assert!(state.has_active_touches());
        assert_eq!(state.active_touch_count(), 1);
        assert_eq!(state.touch_position(1), Some(Vector2::new(200.0, 300.0)));
        assert_eq!(state.touch_origin(1), Some(Vector2::new(200.0, 300.0)));
        assert!(state
            .button_state_specific(
                &InputButton::Touch(1),
                InputId::Touch(DeviceId::dummy())
            )
            .unwrap());

        state.handle_event(touch_event(TouchPhase::Moved, (230.0, 330.0), 1));
        assert_eq!(state.touch_delta(1), Some(Vector2::new(30.0, 30.0)));
        assert_eq!(state.touch_position(1), Some(Vector2::new(230.0, 330.0)));
        assert_eq!(state.touch_origin(1), Some(Vector2::new(200.0, 300.0)));

        state.handle_event(touch_event(TouchPhase::Ended, (230.0, 330.0), 1));
        assert!(!state.has_active_touches());
        assert_eq!(state.touch_position(1), None);
        assert_eq!(state.touch_origin(1), None);
        assert_eq!(state.touch_delta(1), None);
        assert!(!state
            .button_state_specific(
                &InputButton::Touch(1),
                InputId::Touch(DeviceId::dummy())
            )
            .unwrap());
    }

    #[test]
    fn touch_delta_accumulates_across_moves() {
        let mut state = setup_state();
        state.handle_event(touch_event(TouchPhase::Started, (100.0, 100.0), 1));
        state.handle_event(touch_event(TouchPhase::Moved, (120.0, 100.0), 1));
        state.handle_event(touch_event(TouchPhase::Moved, (130.0, 110.0), 1));
        assert_eq!(state.touch_delta(1), Some(Vector2::new(30.0, 10.0)));
    }

    #[test]
    fn reset_deltas_clears_touch_deltas() {
        let mut state = setup_state();
        state.handle_event(touch_event(TouchPhase::Started, (100.0, 100.0), 1));
        state.handle_event(touch_event(TouchPhase::Moved, (130.0, 100.0), 1));
        assert_eq!(state.touch_delta(1), Some(Vector2::new(30.0, 0.0)));

        state.reset_deltas();
        assert_eq!(state.touch_delta(1), None);
        // Positions (touch-down state) survive the delta reset.
        assert!(state.has_active_touches());
    }

    #[test]
    fn touch_gesture_selects_movement_and_look_fingers() {
        let mut state = setup_state();
        // Finger 1: left half (movement)
        state.handle_event(touch_event(TouchPhase::Started, (200.0, 400.0), 1));
        // Finger 2: right half (look)
        state.handle_event(touch_event(TouchPhase::Started, (700.0, 400.0), 2));

        let gesture = state.touch_gesture();
        assert!(gesture.active);
        assert_eq!(gesture.joystick_origin, Some(Vector2::new(200.0, 400.0)));
        assert_eq!(gesture.joystick_position, Some(Vector2::new(200.0, 400.0)));

        // Move movement finger up-right -> forward + strafe right.
        state.handle_event(touch_event(TouchPhase::Moved, (240.0, 370.0), 1));
        // Move look finger right.
        state.handle_event(touch_event(TouchPhase::Moved, (730.0, 400.0), 2));

        let gesture = state.touch_gesture();
        assert!(gesture.move_vector.x > 0.0);
        assert!(gesture.move_vector.y > 0.0);
        assert_eq!(gesture.look_delta, Vector2::new(30.0, 0.0));

        // When both fingers are on the same half, the lowest id wins.
        let mut state = setup_state();
        state.handle_event(touch_event(TouchPhase::Started, (100.0, 300.0), 5));
        state.handle_event(touch_event(TouchPhase::Started, (300.0, 400.0), 3));
        let gesture = state.touch_gesture();
        assert_eq!(gesture.joystick_origin, Some(Vector2::new(300.0, 400.0)));
    }

    #[test]
    fn touch_gesture_no_surface_size_is_inactive() {
        let state = InputState::new();
        let gesture = state.touch_gesture();
        assert!(!gesture.active);
        assert!(gesture.joystick_origin.is_none());
    }
}
