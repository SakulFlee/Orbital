use hashbrown::{hash_map::Entry, HashMap, HashSet};
use log::error;
use winit::{
    event::{DeviceId, ElementState, MouseScrollDelta},
    platform::scancode::PhysicalKeyExtScancode,
};

use crate::{app::InputEvent, error::Error};

use super::{
    action::Action,
    binding::InputBinding,
    frame::InputFrame,
    layout::{Layout, GLOBAL_LAYOUT},
};

/// Automatically handles any input events send from winit and matches them
/// against defined input layouts.
/// Any input even will be looked up, per [DeviceId], and the corresponding
/// [Action] will get triggered.
///
/// Any triggered [Action] inside [InputFrame] are grouped.
/// By default, any [DeviceId] will be added to the [GLOBAL_LAYOUT] and handled
/// globally.
/// However, if needed (e.g. for "local couch co-op") a given [DeviceId] can be
/// moved into it's own group like `"Player #1"`.
/// Any triggered [Action] _by this [DeviceId]_ then will be marked by the
/// specified group, instead of the [GLOBAL_LAYOUT].
#[derive(Debug)]
pub struct InputManager {
    /// Maps a given [DeviceId] to a [Layout]
    device_layout_mapping: HashMap<Layout, HashSet<DeviceId>>,
    /// Maps a given [Layout] to their [Action] mappings.  
    /// Internally, maps [InputBinding] to a given [Action].
    layout_action_mapping: HashMap<Layout, HashMap<InputBinding, Action>>,
    /// The current [InputFrame] to be collected and eventually send off.  
    current_input_frame: InputFrame,
    /// The axis threshold defines when an axis change is considered an input
    /// and not some stick drift or similar.
    axis_threshold: f32,
}

impl InputManager {
    pub fn new() -> Self {
        let mut device_layout_mapping = HashMap::new();
        device_layout_mapping.insert(GLOBAL_LAYOUT, HashSet::new());

        let mut layout_action_mapping = HashMap::new();
        layout_action_mapping.insert(GLOBAL_LAYOUT, HashMap::new());

        Self {
            device_layout_mapping,
            layout_action_mapping,
            current_input_frame: InputFrame::new(),
            axis_threshold: 0.15,
        }
    }

    /// Takes the current [InputFrame] and places a new fresh/reset one
    /// into the same variable.
    ///
    /// This should be called before every update cycle.
    pub fn take_input_frame_and_reset(&mut self) -> InputFrame {
        std::mem::replace(&mut self.current_input_frame, InputFrame::new())
    }

    /// Changes the threshold at which an axis change triggers an [Action].  
    /// The default setting should be sensitive enough for most scenarios.
    pub fn change_axis_threshold(&mut self, threshold: f32) {
        self.axis_threshold = threshold;
    }

    pub fn find_device_layout(&self, device_id: &DeviceId) -> Option<Layout> {
        self.device_layout_mapping
            .iter()
            .find(|(_, v)| v.contains(device_id))
            .map(|(k, _)| k)
            .copied()
    }

    pub fn register_layout(&mut self, layout: Layout, bindings: HashMap<InputBinding, Action>) {
        let entry = self.layout_action_mapping.entry(layout);
        let layout_set = entry.or_default();

        layout_set.extend(bindings);
    }

    pub fn add_binding_to_layout(&mut self, layout: Layout, binding: InputBinding, action: Action) {
        let entry = self.layout_action_mapping.entry(layout);
        let layout_set = entry.or_default();

        layout_set.insert(binding, action);
    }

    pub fn remove_layout(&mut self, layout: Layout) {
        self.layout_action_mapping.remove(layout);
    }

    pub fn remove_binding_from_layout(&mut self, layout: Layout, binding: &InputBinding) {
        let entry = self.layout_action_mapping.entry(layout);
        match entry {
            Entry::Occupied(mut map) => {
                map.get_mut().remove(binding);
            }
            // Skip if it doesn't exist ... nothing to remove.
            _ => (),
        }
    }

    /// Registers a [DeviceId] to the [GLOBAL_LAYOUT] group.  
    /// Use [Self::move_device_layout] to move a given [DeviceId] into a
    /// different [Layout].
    pub fn register_device(&mut self, device_id: &DeviceId) {
        if let Some(device_set) = self.device_layout_mapping.get_mut(GLOBAL_LAYOUT) {
            device_set.insert(*device_id);
        } else {
            error!("Global Layout '{GLOBAL_LAYOUT}' does not exist!");
        }
    }

    /// Unregisters a [DeviceId] from **any** [Layout].
    pub fn unregister_device(&mut self, device_id: &DeviceId) {
        self.device_layout_mapping
            .values_mut()
            .for_each(|x| x.retain(|id| device_id == id));
    }

    /// Moves a given device ([DeviceId]) into a [Layout].
    ///
    /// # Failures
    /// Fails if the given [DeviceId] does not exist.
    pub fn move_device_layout(
        &mut self,
        device_id: &DeviceId,
        layout: Layout,
    ) -> Result<(), Error> {
        let mut found = false;
        self.device_layout_mapping.values_mut().for_each(|x| {
            if x.remove(device_id) {
                found = true;
            }
        });

        if !found {
            error!(
                "Attempting to move device layout when DeviceID '{:?}' does not exist!",
                device_id
            );
            return Err(Error::InputDeviceNotFound(*device_id));
        }

        let entry = self.device_layout_mapping.entry(layout);
        let device_set = entry.or_insert(HashSet::new());

        device_set.insert(*device_id);

        Ok(())
    }

    /// Moves all devices ([DeviceId]s) listed under a given [Layout] into
    /// another [Layout].
    ///
    /// # Failure
    /// Fails, if the `source` [Layout] doesn't exist.  
    /// A missing `to` [Layout] will simply be created.
    pub fn move_all_devices_in_layout(&mut self, from: &Layout, to: &Layout) -> Result<(), Error> {
        // Remove all the devices, including the layout name, from the source.
        // Fails if the source doesn't exist.
        // Keeping an empty entry doesn't make much sense here, thus removing.
        let source_devices = self
            .device_layout_mapping
            .remove(from)
            .ok_or(Error::InputLayoutNotFound(from))?;

        // Ensure destination entry exists and if not insert one
        let destination_entry = self.device_layout_mapping.entry(to);
        let destination_set = destination_entry.or_default();

        // Finally, extend the destination by all devices in source
        destination_set.extend(source_devices);

        Ok(())
    }

    fn check_and_get_device_layout(&self, device_id: &DeviceId) -> Result<Layout, Error> {
        self.find_device_layout(device_id)
            .ok_or(Error::InputDeviceNotFound(device_id.clone()))
    }

    fn check_and_get_layout_bindings(
        &self,
        layout: &Layout,
    ) -> Result<&HashMap<InputBinding, Action>, Error> {
        self.layout_action_mapping
            .get(layout)
            .ok_or(Error::InputLayoutNotFound(layout))
    }

    pub fn handle_input_event(&mut self, input_event: InputEvent) -> Result<(), Error> {
        match input_event {
            InputEvent::Added { device_id } => {
                self.register_device(&device_id);
            }
            InputEvent::Removed { device_id } => {
                self.unregister_device(&device_id);
            }
            InputEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic: _,
            } => {
                if event.state != ElementState::Pressed {
                    // Skip non presses
                    return Ok(());
                }

                let layout = self.check_and_get_device_layout(&device_id)?;
                let input_bindings = self.check_and_get_layout_bindings(&layout)?;

                let scancode = event.physical_key.to_scancode().ok_or(
                    Error::PhysicalKeyToScanCodeConversionError(event.physical_key),
                )?;
                if let Some(action) = input_bindings
                    .iter()
                    .find(|(k, _)| {
                        if let InputBinding::KeyboardKey {
                            key: expected_scancode,
                        } = k
                        {
                            return *expected_scancode == scancode;
                        }

                        false
                    })
                    .map(|(_, v)| v)
                    .cloned()
                {
                    let entry = self.current_input_frame.actions.entry(layout);
                    let set = entry.or_default();
                    set.insert(&action);
                }
            }
            InputEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                if state != ElementState::Pressed {
                    // Skip non presses
                    return Ok(());
                }

                let layout = self.check_and_get_device_layout(&device_id)?;
                let input_bindings = self.check_and_get_layout_bindings(&layout)?;

                if let Some(action) = input_bindings
                    .iter()
                    .find(|(k, _)| {
                        if let InputBinding::MouseButton {
                            button: expected_button,
                        } = k
                        {
                            return *expected_button == button;
                        }

                        false
                    })
                    .map(|(_, v)| v)
                    .cloned()
                {
                    let entry = self.current_input_frame.actions.entry(layout);
                    let set = entry.or_default();
                    set.insert(&action);
                }
            }
            InputEvent::CursorEntered { device_id: _ } => {
                self.current_input_frame.cursor_inside_window = true;
            }
            InputEvent::CursorLeft { device_id: _ } => {
                self.current_input_frame.cursor_inside_window = false;
            }
            InputEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.current_input_frame.cursor_position = position;
            }
            InputEvent::MouseMotion {
                device_id: _,
                delta: _,
            } => {
                // Skipped! Using [InputEvent::CursorMoved] instead!
            }
            InputEvent::MouseWheel {
                device_id,
                delta,
                phase: _,
            } => {
                self.current_input_frame.mouse_scroll = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x, y),
                    MouseScrollDelta::PixelDelta(x) => (-x.x as f32, -x.y as f32),
                };

                match if self.current_input_frame.mouse_scroll.0 >= self.axis_threshold {
                    Some(InputBinding::MouseWheel {
                        is_x: true,
                        is_increasing: true,
                    })
                } else if self.current_input_frame.mouse_scroll.0 <= -self.axis_threshold {
                    Some(InputBinding::MouseWheel {
                        is_x: true,
                        is_increasing: false,
                    })
                } else if self.current_input_frame.mouse_scroll.1 >= self.axis_threshold {
                    Some(InputBinding::MouseWheel {
                        is_x: false,
                        is_increasing: true,
                    })
                } else if self.current_input_frame.mouse_scroll.1 <= -self.axis_threshold {
                    Some(InputBinding::MouseWheel {
                        is_x: false,
                        is_increasing: false,
                    })
                } else {
                    None
                } {
                    Some(target_binding) => {
                        let layout = self.check_and_get_device_layout(&device_id)?;
                        let bindings = self.check_and_get_layout_bindings(&layout)?;

                        if let Some(action) = bindings
                            .iter()
                            .find(|(k, _)| {
                                if let InputBinding::MouseWheel {
                                    is_x: target_is_x,
                                    is_increasing: target_is_increasing,
                                } = target_binding
                                {
                                    if let InputBinding::MouseWheel {
                                        is_x,
                                        is_increasing,
                                    } = k
                                    {
                                        return *is_x == target_is_x
                                            && *is_increasing == target_is_increasing;
                                    }
                                }

                                false
                            })
                            .map(|(_, v)| v)
                            .cloned()
                        {
                            let entry = self.current_input_frame.actions.entry(layout);
                            let set = entry.or_default();
                            set.insert(&action);
                        }
                    }
                    None => (),
                }
            }
            InputEvent::AxisMotion {
                device_id,
                axis,
                value,
            } => {
                let layout = self.check_and_get_device_layout(&device_id)?;
                let input_bindings = self.check_and_get_layout_bindings(&layout)?;

                let increasing = if value as f32 >= self.axis_threshold {
                    true
                } else if value as f32 <= -self.axis_threshold {
                    false
                } else {
                    return Ok(());
                };

                if let Some(action) = input_bindings
                    .iter()
                    .find(|(k, _)| {
                        if let InputBinding::GamepadAxis {
                            axis_id,
                            increasing: k_increasing,
                        } = k
                        {
                            return axis == *axis_id && increasing == *k_increasing;
                        }

                        false
                    })
                    .map(|(_, v)| v)
                    .cloned()
                {
                    let entry = self.current_input_frame.actions.entry(layout);
                    let set = entry.or_default();
                    set.insert(&action);

                    let entry = self.current_input_frame.axis.entry(layout);
                    let axis_map = entry.or_default();
                    axis_map.insert(&action, value);
                }
            }
            InputEvent::Button {
                device_id,
                button,
                state,
            } => {
                if state != ElementState::Pressed {
                    // Skip non presses
                    return Ok(());
                }

                let layout = self.check_and_get_device_layout(&device_id)?;
                let input_bindings = self.check_and_get_layout_bindings(&layout)?;

                if let Some(action) = input_bindings
                    .iter()
                    .find(|(k, _)| {
                        if let InputBinding::GamepadButton {
                            button: expected_button,
                        } = k
                        {
                            return *expected_button == button;
                        }

                        false
                    })
                    .map(|(_, v)| v)
                    .cloned()
                {
                    let entry = self.current_input_frame.actions.entry(layout);
                    let set = entry.or_default();
                    set.insert(&action);
                }
            }
        }

        Ok(())
    }
}
