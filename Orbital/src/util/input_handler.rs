use gilrs::{Axis, Button};
use hashbrown::{HashMap, HashSet};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, MouseScrollDelta},
    keyboard::PhysicalKey,
};

use crate::app::{InputEvent, WINDOW_HALF_SIZE};

pub type Action = &'static str;

#[derive(Debug, Default)]
pub struct InputHandler {
    // --- Settings ---
    /// Sets the [Axis] threshold.  
    /// This is done to prevent issues like "stick drift" or "micro
    /// stuttering".  
    /// Essentially, this means you have to move the [Axis] at least by the
    /// set amount, before it is considered triggered.
    ///
    /// A smaller value gives you more accuracy, but might cause issues with
    /// well used or cheap controllers.  
    /// Essentially:  
    /// The range is 0.0 to 1.0 (or -1.0 to 1.0 for JoySticks).  
    /// 0.0 ==   0% triggered  
    /// 1.0 == 100% triggered  
    ///
    /// Setting this value to 0.1 (== 10%), would result in a motion range from
    /// 0.1 to 1.0. In percent: 10% to 100%.  
    /// Meaning, we only can use 90% (100% - 10%) of the [Axis].
    ///
    /// Setting this value to 0.5 (== 50%), would result in a motion range from
    /// 0.5 to 1.0. In percent: 50% to 100%.  
    /// Meaning, we can only use 50% (100% - 50%) of the [Axis].
    ///
    /// Check [Self::DEFAULT_AXIS_THRESHOLD] for a good standard value which
    /// should work on most controllers, while allowing for a lot of movement.
    axis_threshold: f32,
    // --- Mappings ---
    /// Mapping for binding [PhysicalKey]s (Keyboard Buttons) to [Action]s.  
    /// This is a **digital/binary** mapping!
    keyboard_mapping: HashMap<PhysicalKey, Action>,
    /// Mapping for binding [MouseButton]s to [Action]s.  
    /// This is a **digital/binary** mapping!
    mouse_button_mapping: HashMap<MouseButton, Action>,
    /// Mapping for binding GamePad [Button]s to [Action]s.  
    /// This is a **digital/binary** mapping!
    gamepad_button_mapping: HashMap<Button, Action>,
    /// Mapping for binding GamePad [Axis]s (like JoySticks or Triggers) to
    /// [Action]s.  
    /// This is a **analogue/variable** mapping!
    gamepad_axis_mapping: HashMap<Axis, Action>,
    // --- Actions ---
    /// Contains any **triggered digital/binary [Action]s**.  
    /// If an [Action] stops being triggered, it should be removed from here.
    triggered_actions: HashSet<Action>,
    /// Contains any **triggered analogue/variable [Action]s**.  
    /// If an [Action] stops being triggered (i.e. within the
    /// [Self::axis_threshold]), it should be removed from here.
    triggered_action_axis: HashMap<Action, f32>,
    // --- Other Input Data ---
    /// Stores the current mouse position.
    mouse_position: PhysicalPosition<f64>,
    /// Stores the current mouse scrolling values.
    mouse_scrolling: (f32, f32),
}

impl InputHandler {
    pub const DEFAULT_AXIS_THRESHOLD: f32 = 0.15;

    pub fn new() -> Self {
        Self {
            axis_threshold: Self::DEFAULT_AXIS_THRESHOLD,
            ..Default::default()
        }
    }

    pub fn set_axis_threshold(&mut self, axis_threshold: f32) {
        self.axis_threshold = axis_threshold;
    }

    pub fn get_cursor_position(&self) -> PhysicalPosition<f64> {
        self.mouse_position
    }

    pub fn calculate_view_change_from_axis_and_cursor(
        &self,
        action_x_axis: &'static str,
        action_y_axis: &'static str,
    ) -> (bool, f32, f32) {
        if let Some(axis_result) =
            self.calculate_view_change_from_axis(action_x_axis, action_y_axis)
        {
            return (true, axis_result.0, axis_result.1);
        } else {
            let cursor_result = self.calculate_view_change_from_cursor();
            return (false, cursor_result.0, cursor_result.1);
        }
    }

    pub fn calculate_view_change_from_axis(
        &self,
        action_x_axis: &'static str,
        action_y_axis: &'static str,
    ) -> Option<(f32, f32)> {
        let option_x = self.get_only_axis(action_x_axis);
        let option_y = self.get_only_axis(action_y_axis);

        if option_x.is_some() && option_y.is_some() {
            return Some((option_x.unwrap(), option_y.unwrap()));
        } else if option_x.is_some() || option_y.is_some() {
            return Some((
                option_x.or(Some(0.0)).unwrap(),
                option_y.or(Some(0.0)).unwrap(),
            ));
        } else {
            return None;
        }
    }

    /// Calculates the change of `yaw` and `pitch` based on mouse movement.  
    /// Assumes the mouse cursor deviated from the center of the window
    /// (see [WINDOW_HALF_SIZE]) and will produce false results otherwise.
    ///
    /// Returns the calculated **radial** change.
    /// First, `yaw`, last, `pitch`.
    ///
    /// ⚠️ Will make use of [WINDOW_HALF_SIZE] which is potentially dangerous
    /// outside of Rust!
    pub fn calculate_view_change_from_cursor(&self) -> (f32, f32) {
        let cursor_position: PhysicalPosition<i32> = self.get_cursor_position().cast();
        let window_half_size = unsafe { WINDOW_HALF_SIZE };

        let yaw_change = (cursor_position.x - window_half_size.0) as f32;
        let pitch_change = (window_half_size.1 - cursor_position.y) as f32;

        (yaw_change, pitch_change)
    }

    pub fn get_mouse_scrolling(&self) -> (f32, f32) {
        self.mouse_scrolling
    }

    pub fn register_keyboard_mapping(&mut self, key: PhysicalKey, action: &'static str) {
        self.keyboard_mapping.insert(key, action);
    }

    pub fn register_mouse_button_mapping(&mut self, button: MouseButton, action: &'static str) {
        self.mouse_button_mapping.insert(button, action);
    }

    pub fn register_gamepad_button_mapping(&mut self, button: Button, action: &'static str) {
        self.gamepad_button_mapping.insert(button, action);
    }

    pub fn register_gamepad_axis_mapping(&mut self, axis: Axis, action: &'static str) {
        self.gamepad_axis_mapping.insert(axis, action);
    }

    /// Checks whether a button action is triggered or not.
    ///
    /// # Returns
    /// `true`, if triggered.
    /// `false`, if not triggered.
    pub fn is_triggered(&self, action: &'static str) -> bool {
        self.triggered_actions.contains(action)
    }

    /// Checks if an axis value is registered and returns it.
    /// Otherwise, falls back to [Self::get_button_axis].
    ///
    /// # Returns
    /// `Some(-1.0..=1.0I)`, if the given axis has registered a value.
    /// `Some(1.0)`, if the positive button axis is triggered.
    /// `Some(-1.0)`, if the negative button axis is triggered.
    /// `None`, if neither is triggered.
    ///
    /// ⚠️ Axis are preferred over button axis.
    /// ⚠️ Positive button axis are favoured over negative.
    /// This means, if an axis and a button is triggering an action,
    /// the axis value will be favoured over any button presses.
    pub fn get_dynamic_axis(
        &self,
        axis_action: &'static str,
        positive_action: &'static str,
        negative_action: &'static str,
    ) -> Option<f32> {
        let axis = self.triggered_action_axis.get(axis_action).cloned(); // TODO
        if axis.is_some() {
            return axis;
        }

        self.get_button_axis(positive_action, negative_action)
    }

    /// Converts two button presses into an axis value like
    /// [Self::get_only_axis]. Normally, an axis ranges from
    /// -1.0 .. 0.0 .. 0.0, but to achieve the same with buttons
    /// we require two buttons. Thus, two actions!
    ///
    /// # Returns
    /// `Some(1.0)`, if the positive action is triggered.
    /// `Some(-1.0)`, if the negative action is triggered.
    /// `None`, if neither action is triggered.
    ///
    /// ⚠️ Positive triggers are favoured over negative ones!
    /// Meaning, if both are triggered, the positive one will
    /// return `1.0` first, before the negative can return `-1.0`.
    pub fn get_button_axis(
        &self,
        positive_action: &'static str,
        negative_action: &'static str,
    ) -> Option<f32> {
        if self.triggered_actions.contains(positive_action) {
            return Some(1.0);
        } else if self.triggered_actions.contains(negative_action) {
            return Some(-1.0);
        }

        None
    }

    /// Considers **only** actual axis values like from a joystick.  
    ///
    /// # Returns
    /// `None`, if there is no axis value registered.
    /// This can happen if the given axis value is below [Self::axis_threshold].
    ///
    /// `Some<f32>`, if the axis has a value registered.
    /// Should be normalized within -1.0 .. 0.0 .. 1.0, but the **raw**
    /// axis value is taken.
    /// Thus, "weird" controllers potentially could have weird values here.
    pub fn get_only_axis(&self, action: &'static str) -> Option<f32> {
        self.triggered_action_axis.get(action).cloned()
    }

    /// Handles [InputEvent]s by checking for assigned input mappings
    /// and matching them against [InputDescriptor]s.
    ///
    /// Any analogue (axis) inputs will be prioritized over digital (binary)
    /// inputs. What this means is, say you have a controller and a keyboard
    /// plugged in. There is an input binding for both mapping the same action.
    /// The controller uses a joystick (== axis), the keyboard a button press
    /// (== binary). Now imagine both are triggered. In this scenario we prefer
    /// using the **more accurate** axis over the binary input from the
    /// keyboard. Effectively meaning: We ignore keyboard inputs (and e.g.
    /// controller button pressed) if the same action is triggered by a
    /// controller joystick (or axis of any kind).
    pub fn handle_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::KeyboardButton {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let Some(action) = self.keyboard_mapping.iter().find_map(|(k, v)| {
                    if event.physical_key.eq(k) {
                        Some(*v)
                    } else {
                        None
                    }
                }) {
                    match event.state {
                        // If pressed, trigger the action
                        ElementState::Pressed => {
                            self.triggered_actions.insert(action);
                        }
                        // If no longer pressed, remove the action
                        ElementState::Released => {
                            self.triggered_actions.remove(action);
                        }
                    }
                }
            }
            InputEvent::MouseButton {
                device_id: _,
                state,
                button,
            } => {
                if let Some(action) = self.mouse_button_mapping.iter().find_map(|(k, v)| {
                    if k.eq(button) {
                        Some(v)
                    } else {
                        None
                    }
                }) {
                    match state {
                        ElementState::Pressed => {
                            self.triggered_actions.insert(action);
                        }
                        ElementState::Released => {
                            self.triggered_actions.remove(action);
                        }
                    }
                }
            }
            InputEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                self.mouse_scrolling = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                    MouseScrollDelta::PixelDelta(delta) => (delta.x as f32, delta.y as f32),
                };
            }
            InputEvent::MouseMoved {
                device_id: _,
                position,
            } => {
                self.mouse_position = *position;
            }
            InputEvent::GamepadButton {
                gamepad_id: _,
                button,
                pressed,
            } => {
                if let Some(action) = self.gamepad_button_mapping.iter().find_map(|(k, v)| {
                    if k == button {
                        Some(*v)
                    } else {
                        None
                    }
                }) {
                    match pressed {
                        true => {
                            self.triggered_actions.insert(action);
                        }
                        false => {
                            self.triggered_actions.remove(action);
                        }
                    }
                }
            }
            InputEvent::GamepadAxis {
                gamepad_id: _,
                axis,
                value,
            } => {
                if let Some(action) = self.gamepad_axis_mapping.get(axis) {
                    if value.gt(&self.axis_threshold) {
                        self.triggered_action_axis.insert(*action, *value);
                    } else if value.lt(&-self.axis_threshold) {
                        self.triggered_action_axis.insert(*action, *value);
                    } else {
                        self.triggered_action_axis.remove(*action);
                    }
                }
            }
            _ => (),
        }
    }
}
