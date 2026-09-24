//! iOS gamepad provider: polls the GameController framework.
//!
//! The winit fork carries Android's gamepad translation as device
//! events, but winit does not link GameController on iOS, so the
//! engine talks to `GCController` itself — mirroring how desktop
//! polls gilrs. Controllers are enumerated once per frame, diffs
//! against the previous report become the same backend-neutral
//! [`InputEvent`]s every other backend produces, and no Objective-C
//! object outlives the poll.

use std::collections::HashMap;

use cgmath::{InnerSpace, Vector2, Zero};
use objc2_game_controller::{GCController, GCControllerButtonInput, GCControllerDirectionPad};

use orbital_input::{GamepadButton, InputAxis, InputEvent};

/// Radial dead zone applied to both sticks: deflections below this
/// magnitude report as centered, filtering out resting drift.
const STICK_DEADZONE: f64 = 0.15;

/// Per-controller last-reported state, used to emit only changes.
struct Pad {
    id: u32,
    buttons: HashMap<GamepadButton, bool>,
    left_stick: Vector2<f64>,
    right_stick: Vector2<f64>,
    /// `(left, right)` trigger values in `0.0..=1.0`.
    triggers: (f64, f64),
}

impl Pad {
    fn new(id: u32) -> Self {
        Self {
            id,
            buttons: HashMap::new(),
            left_stick: Vector2::zero(),
            right_stick: Vector2::zero(),
            triggers: (0.0, 0.0),
        }
    }
}

pub struct IosGamepad {
    next_id: u32,
    /// Keyed by controller object address; values are engine-side only.
    pads: HashMap<usize, Pad>,
}

impl IosGamepad {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            pads: HashMap::new(),
        }
    }

    /// Read every connected controller once and return the changes as
    /// input events (connects, disconnects, button presses/releases,
    /// and axis movements).
    pub fn poll(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();
        let controllers = unsafe { GCController::controllers() };

        let mut alive = HashMap::new();
        for index in 0..controllers.count() {
            let controller = controllers.objectAtIndex(index);
            let key = std::ptr::from_ref(&*controller).addr();

            // Only the standard (extended) profile maps onto the
            // engine's semantic button/axis set; MiFi-only profiles
            // such as the old GCGamepad gamepad are skipped.
            let Some(profile) = (unsafe { controller.extendedGamepad() }) else {
                continue;
            };

            if !self.pads.contains_key(&key) {
                let id = self.next_id;
                self.next_id += 1;
                self.pads.insert(key, Pad::new(id));
                events.push(InputEvent::GamepadConnected { gamepad_id: id });
                // The diff below then reports the initial state: every
                // currently-pressed button and any non-centered stick
            }
            let pad = &mut self.pads[&key];
            poll_profile(&profile, pad, &mut events);
            alive.insert(key);
        }

        let disconnected: Vec<usize> = self
            .pads
            .keys()
            .filter(|key| !alive.contains_key(key))
            .copied()
            .collect();
        for key in disconnected {
            if let Some(pad) = self.pads.remove(&key) {
                events.push(InputEvent::GamepadDisconnected { gamepad_id: pad.id });
            }
        }

        events
    }
}

fn poll_profile(
    profile: &objc2_game_controller::GCExtendedGamepad,
    pad: &mut Pad,
    events: &mut Vec<InputEvent>,
) {
    // SAFETY: All reads are plain getters on a live profile object
    // reached through the controller enumerated this poll.
    unsafe {
        let dpad = profile.dpad();
        poll_button(events, pad, &dpad.up(), GamepadButton::DPadUp);
        poll_button(events, pad, &dpad.down(), GamepadButton::DPadDown);
        poll_button(events, pad, &dpad.left(), GamepadButton::DPadLeft);
        poll_button(events, pad, &dpad.right(), GamepadButton::DPadRight);

        // Xbox-layout face buttons: A bottom (South), B right (East),
        // X left (West), Y top (North).
        poll_button(events, pad, &profile.buttonA(), GamepadButton::South);
        poll_button(events, pad, &profile.buttonB(), GamepadButton::East);
        poll_button(events, pad, &profile.buttonX(), GamepadButton::West);
        poll_button(events, pad, &profile.buttonY(), GamepadButton::North);

        poll_button(
            events,
            pad,
            &profile.leftShoulder(),
            GamepadButton::LeftTrigger,
        );
        poll_button(
            events,
            pad,
            &profile.rightShoulder(),
            GamepadButton::RightTrigger,
        );

        poll_button(events, pad, &profile.buttonMenu(), GamepadButton::Start);
        if let Some(options) = profile.buttonOptions() {
            poll_button(events, pad, &options, GamepadButton::Select);
        }
        if let Some(thumb) = profile.leftThumbstickButton() {
            poll_button(events, pad, &thumb, GamepadButton::LeftThumb);
        }
        if let Some(thumb) = profile.rightThumbstickButton() {
            poll_button(events, pad, &thumb, GamepadButton::RightThumb);
        }

        // Triggers are button elements that also report an analog
        // value: the press becomes the digital button, the value the
        // trigger axis (left on x, right on y, matching gilrs).
        poll_trigger(events, pad, &profile.leftTrigger(), 0);
        poll_trigger(events, pad, &profile.rightTrigger(), 1);

        poll_stick(events, pad, &profile.leftThumbstick(), false);
        poll_stick(events, pad, &profile.rightThumbstick(), true);
    }
}

fn poll_button(
    events: &mut Vec<InputEvent>,
    pad: &mut Pad,
    input: &GCControllerButtonInput,
    button: GamepadButton,
) {
    let pressed = unsafe { input.isPressed() };
    if pad.buttons.get(&button).copied().unwrap_or(false) != pressed {
        pad.buttons.insert(button, pressed);
        events.push(InputEvent::GamepadButton {
            gamepad_id: pad.id,
            button,
            pressed,
        });
    }
}

fn poll_trigger(
    events: &mut Vec<InputEvent>,
    pad: &mut Pad,
    input: &GCControllerButtonInput,
    side: usize,
) {
    let value = unsafe { input.value() } as f64;
    let previous = if side == 0 {
        pad.triggers.0
    } else {
        pad.triggers.1
    };
    if value != previous {
        if side == 0 {
            pad.triggers.0 = value;
            events.push(InputEvent::GamepadAxis {
                gamepad_id: pad.id,
                axis: InputAxis::GamepadTrigger,
                delta: Vector2::new(value, 0.0),
            });
        } else {
            pad.triggers.1 = value;
            events.push(InputEvent::GamepadAxis {
                gamepad_id: pad.id,
                axis: InputAxis::GamepadTrigger,
                delta: Vector2::new(0.0, value),
            });
        }
    }

    let (button, digital) = if side == 0 {
        (GamepadButton::LeftTrigger2, input)
    } else {
        (GamepadButton::RightTrigger2, input)
    };
    let pressed = unsafe { digital.isPressed() };
    if pad.buttons.get(&button).copied().unwrap_or(false) != pressed {
        pad.buttons.insert(button, pressed);
        events.push(InputEvent::GamepadButton {
            gamepad_id: pad.id,
            button,
            pressed,
        });
    }
}

fn poll_stick(
    events: &mut Vec<InputEvent>,
    pad: &mut Pad,
    dpad: &GCControllerDirectionPad,
    right: bool,
) {
    let x = unsafe { dpad.xAxis().value() } as f64;
    let y = unsafe { dpad.yAxis().value() } as f64;
    let raw = Vector2::new(x, y);
    let position = if raw.magnitude() < STICK_DEADZONE {
        Vector2::zero()
    } else {
        raw
    };

    let (previous, axis) = if right {
        (&mut pad.right_stick, InputAxis::GamepadRightStick)
    } else {
        (&mut pad.left_stick, InputAxis::GamepadLeftStick)
    };

    // Components are reported separately (x as `(x, 0)`, y as
    // `(0, y)`) exactly like the gilrs and device-event conversions.
    if position.x != previous.x {
        previous.x = position.x;
        events.push(InputEvent::GamepadAxis {
            gamepad_id: pad.id,
            axis,
            delta: Vector2::new(position.x, 0.0),
        });
    }
    if position.y != previous.y {
        previous.y = position.y;
        events.push(InputEvent::GamepadAxis {
            gamepad_id: pad.id,
            axis,
            delta: Vector2::new(0.0, position.y),
        });
    }
}
