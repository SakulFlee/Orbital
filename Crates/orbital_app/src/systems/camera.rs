//! Camera controller system — WASD movement + mouse look, gamepad (left
//! stick move / right stick look), with automatic touch-screen fallback
//! (virtual joystick + drag-to-look).
//!
//! Reads `DeltaTime` and `InputSnapshot` resources, writes `Position` and
//! `Rotation` components on entities with a camera.

use cgmath::{Rad, Vector2};
use orbital_camera::{CameraDescriptor as CameraLimits, ProjectionMode};
use orbital_ecs::{ComponentAccess, Res, System, World};
use orbital_ecs_bridge::{
    CameraDescriptor, CursorGrabState, DeltaTime, IcedCapturedMouseDrag, InputSnapshot, Position,
    Rotation,
};
use orbital_input::{InputAxis, InputButton, InputState};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::touch_controls;

/// How the orthographic camera maps W/S.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrthographicForwardMode {
    /// W/S zooms by changing `ortho_height`.
    #[default]
    Dolly,
    /// W/S moves the camera along its view direction.
    Depth,
}

/// Runtime camera-controller preferences.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraControlSettings {
    /// W/S behavior while the active camera is orthographic.
    pub orthographic_forward: OrthographicForwardMode,
    /// Whether the controller processes input.
    pub enabled: bool,
    /// Movement speed in world units per second at full input.
    pub move_speed: f32,
    /// Movement multiplier while Shift is held.
    pub sprint_multiplier: f32,
    /// Look sensitivity for normalized mouse/gamepad input.
    pub look_sensitivity: f32,
    /// Look smoothing time constant in seconds. `0` is immediate.
    pub smoothing: f32,
    /// Maximum absolute pitch in radians.
    pub pitch_limit: f32,
    /// Invert horizontal look.
    pub invert_x: bool,
    /// Invert vertical look.
    pub invert_y: bool,
    /// Orthographic Q/E and dolly zoom speed.
    pub ortho_zoom_speed: f32,
}

impl Default for CameraControlSettings {
    fn default() -> Self {
        Self {
            orthographic_forward: OrthographicForwardMode::Dolly,
            enabled: true,
            move_speed: 5.0,
            sprint_multiplier: 2.0,
            look_sensitivity: 1.5,
            smoothing: 0.0,
            pitch_limit: CameraLimits::SAFE_FRAC_PI_2,
            invert_x: false,
            invert_y: false,
            ortho_zoom_speed: 2.5,
        }
    }
}

const ORTHO_ZOOM_SPEED: f32 = 2.5;
const ORTHO_MIN_HEIGHT: f32 = 0.1;
const ORTHO_MAX_HEIGHT: f32 = 10000.0;
/// Angular speed (rad/s) at full right-stick deflection. Unlike mouse deltas
/// (impulses, already frame-sized), a stick reports its position, so it is
/// scaled by `dt` into an angular velocity.
const STICK_LOOK_SPEED: f32 = 2.0 / 1.5;

#[derive(Debug, Clone, Copy)]
struct ControllerState {
    move_input: Vector2<f32>,
    look_input: Vector2<f32>,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            move_input: Vector2::new(0.0, 0.0),
            look_input: Vector2::new(0.0, 0.0),
        }
    }
}

/// Camera controller system: WASD movement + mouse look + gamepad sticks.
///
/// When any finger is touching the screen (mobile), it automatically switches
/// to touch controls instead ([`sys_touch_camera_controller`]).
///
/// Mouse look is skipped when [`CursorGrabState`] is `false` (cursor visible),
/// so the user can interact with UI while the camera holds position. It is
/// also skipped while iced owns the current primary-button drag (the press
/// started on a widget), so dragging UI elements never turns the camera.
pub fn sys_camera_controller() -> Box<dyn System> {
    Box::new(CameraControllerSystem::new())
}

struct CameraControllerSystem {
    access: ComponentAccess,
    state: ControllerState,
}

impl CameraControllerSystem {
    fn new() -> Self {
        Self {
            access: ComponentAccess::new()
                .writes::<Position>()
                .writes::<Rotation>(),
            state: ControllerState::default(),
        }
    }
}

impl System for CameraControllerSystem {
    fn name(&self) -> &str {
        "camera_controller"
    }

    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, _commands: &mut orbital_ecs::Commands) {
        let Some(dt) = world.get_resource::<DeltaTime>() else {
            return;
        };
        let Some(input) = world.get_resource::<InputSnapshot>() else {
            return;
        };
        let Some(grab) = world.get_resource::<CursorGrabState>() else {
            return;
        };
        let ui_drag = world
            .get_resource::<IcedCapturedMouseDrag>()
            .map(|drag| drag.0)
            .unwrap_or(false);
        let settings = world
            .get_resource::<CameraControlSettings>()
            .map(|settings| *settings)
            .unwrap_or_default();
        if !settings.enabled {
            return;
        }

        let Some(mut positions) = world.get_component_store_mut::<Position>() else {
            return;
        };
        let Some(mut rotations) = world.get_component_store_mut::<Rotation>() else {
            return;
        };

        let pivot = if positions.dense.len() <= rotations.dense.len() {
            positions.dense.clone()
        } else {
            rotations.dense.clone()
        };

        for eid in pivot {
            let Some(pos_idx) = positions.sparse.get(eid).copied().flatten() else {
                continue;
            };
            let Some(rot_idx) = rotations.sparse.get(eid).copied().flatten() else {
                continue;
            };

            let pos = &mut positions.components[pos_idx];
            let rot = &mut rotations.components[rot_idx];
            if input.0.has_active_touches() {
                apply_touch_controls(dt.0 as f32, &input.0, &settings, &mut self.state, pos, rot);
            } else {
                apply_keyboard_mouse_controls(
                    dt.0 as f32,
                    &input.0,
                    grab.0 && !ui_drag,
                    &settings,
                    &mut self.state,
                    pos,
                    rot,
                );
            }
        }
    }
}

fn smooth(current: f32, target: f32, dt: f32, smoothing: f32) -> f32 {
    if smoothing <= 0.0 || dt <= 0.0 {
        return target;
    }

    let alpha = 1.0 - (-dt / smoothing).exp();
    current + (target - current) * alpha
}

fn smooth_input(
    current: Vector2<f32>,
    target: Vector2<f32>,
    dt: f32,
    smoothing: f32,
) -> Vector2<f32> {
    Vector2::new(
        smooth(current.x, target.x, dt, smoothing),
        smooth(current.y, target.y, dt, smoothing),
    )
}

/// Orthographic camera controller. Adjusts `ortho_height` with Q/E and no-ops
/// on perspective cameras. Pair with [`sys_ortho_pan`] for a complete
/// orthographic controller.
pub fn sys_ortho_zoom(
    dt: Res<DeltaTime>,
    input: Res<InputSnapshot>,
    camera: &mut CameraDescriptor,
) {
    if camera.projection_mode != ProjectionMode::Orthographic {
        return;
    }

    let zoom_in = input
        .0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyQ)))
        .map(|(_, state)| state)
        .unwrap_or(false);
    let zoom_out = input
        .0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyE)))
        .map(|(_, state)| state)
        .unwrap_or(false);
    let zoom = f32::from(zoom_out) - f32::from(zoom_in);
    if zoom != 0.0 {
        let factor = (1.0 + zoom * ORTHO_ZOOM_SPEED * dt.0 as f32).clamp(0.25, 4.0);
        camera.ortho_height =
            (camera.ortho_height * factor).clamp(ORTHO_MIN_HEIGHT, ORTHO_MAX_HEIGHT);
    }
}

/// Orthographic W/S dolly controller.
///
/// When [`CameraControlSettings::orthographic_forward`] is
/// [`OrthographicForwardMode::Dolly`], W and S change `ortho_height` instead
/// of translating along the view axis. The latter has no visible effect in an
/// orthographic projection, so this keeps forward/back movement responsive.
pub fn sys_ortho_dolly() -> Box<dyn System> {
    Box::new(OrthoDollySystem::new())
}

struct OrthoDollySystem {
    access: ComponentAccess,
}

impl OrthoDollySystem {
    fn new() -> Self {
        Self {
            access: ComponentAccess::new()
                .reads::<CameraDescriptor>()
                .writes::<CameraDescriptor>(),
        }
    }
}

impl System for OrthoDollySystem {
    fn name(&self) -> &str {
        "ortho_dolly"
    }

    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, _commands: &mut orbital_ecs::Commands) {
        let Some(settings) = world.get_resource::<CameraControlSettings>() else {
            return;
        };
        if settings.orthographic_forward != OrthographicForwardMode::Dolly {
            return;
        }

        let dt = world
            .get_resource::<DeltaTime>()
            .map(|delta| delta.0 as f32)
            .unwrap_or(0.0);
        let Some(input) = world.get_resource::<InputSnapshot>() else {
            return;
        };

        let zoom_in = input
            .0
            .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyW)))
            .map(|(_, state)| state)
            .unwrap_or(false);
        let zoom_out = input
            .0
            .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyS)))
            .map(|(_, state)| state)
            .unwrap_or(false);
        let zoom = f32::from(zoom_out) - f32::from(zoom_in);
        if zoom == 0.0 || dt <= 0.0 {
            return;
        }

        let Some(store) = world.get_component_store_mut::<CameraDescriptor>() else {
            return;
        };
        let cameras: Vec<usize> = store
            .dense
            .as_slice()
            .iter()
            .filter_map(|&entity_id| store.sparse.get(entity_id).copied().flatten())
            .collect();

        let factor = (1.0 + zoom * settings.ortho_zoom_speed * dt).clamp(0.25, 4.0);
        for index in cameras {
            let camera = &mut store.get_mut_store().components[index];
            if camera.projection_mode != ProjectionMode::Orthographic {
                continue;
            }
            camera.ortho_height =
                (camera.ortho_height * factor).clamp(ORTHO_MIN_HEIGHT, ORTHO_MAX_HEIGHT);
        }
    }
}

/// Orthographic pan/look controller. Use this instead of
/// [`sys_camera_controller`] for an orthographic camera.
pub fn sys_ortho_pan(
    dt: Res<DeltaTime>,
    input: Res<InputSnapshot>,
    grab: Res<CursorGrabState>,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let settings = CameraControlSettings::default();
    let mut state = ControllerState::default();
    if input.0.has_active_touches() {
        apply_touch_ortho_controls(dt.0 as f32, &input.0, &settings, &mut state, pos, rot);
    } else {
        apply_ortho_pan(
            dt.0 as f32,
            &input.0,
            grab.0,
            &settings,
            &mut state,
            pos,
            rot,
        );
    }
}

/// Touch-screen camera controller: left half = virtual joystick (move),
/// right half = drag to look. No-op when no finger is touching.
///
/// A touch-only controller for games that always want touch semantics,
/// regardless of the active input devices.
pub fn sys_touch_camera_controller(
    dt: Res<DeltaTime>,
    input: Res<InputSnapshot>,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let settings = CameraControlSettings::default();
    let mut state = ControllerState::default();
    apply_touch_controls(dt.0 as f32, &input.0, &settings, &mut state, pos, rot);
}

fn apply_touch_controls(
    dt: f32,
    input: &InputState,
    settings: &CameraControlSettings,
    state: &mut ControllerState,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let cfg = touch_controls();
    let gesture = input.touch_gesture_with_config(cfg.joystick_radius, cfg.joystick_deadzone);
    if !gesture.active {
        return;
    }

    let target_move = Vector2::new(gesture.move_vector.x as f32, gesture.move_vector.y as f32);
    let move_input = smooth_input(state.move_input, target_move, dt, settings.smoothing);
    state.move_input = move_input;

    let (forward, right, _up) = rot.forward_right_up();
    let movement = forward * (move_input.y * cfg.move_speed * dt)
        + right * (move_input.x * cfg.move_speed * dt);
    pos.0 += movement;

    // Drag-to-look on the right half. `look_delta` is in raw screen pixels
    // (x = horizontal, y = vertical, positive = down). Touch keeps its own
    // per-pixel sensitivity, while smoothing and pitch limits are shared.
    let target_look = Vector2::new(
        gesture.look_delta.x as f32 * if settings.invert_y { -1.0 } else { 1.0 },
        gesture.look_delta.y as f32 * if settings.invert_x { -1.0 } else { 1.0 },
    );
    let look = smooth_input(state.look_input, target_look, dt, settings.smoothing);
    state.look_input = look;

    let pitch_delta = -look.y * cfg.look_sensitivity;
    rot.rotate_pitch_clamped(Rad(pitch_delta), Rad(settings.pitch_limit));

    rot.rotate_yaw(Rad(-look.x * cfg.look_sensitivity));
}

fn apply_touch_ortho_controls(
    dt: f32,
    input: &InputState,
    settings: &CameraControlSettings,
    state: &mut ControllerState,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let cfg = touch_controls();
    let gesture = input.touch_gesture_with_config(cfg.joystick_radius, cfg.joystick_deadzone);
    if !gesture.active {
        return;
    }

    let target_move = Vector2::new(gesture.move_vector.x as f32, gesture.move_vector.y as f32);
    let move_input = smooth_input(state.move_input, target_move, dt, settings.smoothing);
    state.move_input = move_input;

    let (_, right, up) = rot.forward_right_up();
    pos.0 += right * (move_input.x * cfg.move_speed * dt);
    pos.0 += up * (move_input.y * cfg.move_speed * dt);

    let target_look = Vector2::new(
        gesture.look_delta.x as f32 * if settings.invert_y { -1.0 } else { 1.0 },
        gesture.look_delta.y as f32 * if settings.invert_x { -1.0 } else { 1.0 },
    );
    let look = smooth_input(state.look_input, target_look, dt, settings.smoothing);
    state.look_input = look;
    rot.rotate_pitch_clamped(
        Rad(-look.y * cfg.look_sensitivity),
        Rad(settings.pitch_limit),
    );
    rot.rotate_yaw(Rad(-look.x * cfg.look_sensitivity));
}

fn apply_ortho_pan(
    dt: f32,
    input: &InputState,
    cursor_grabbed: bool,
    settings: &CameraControlSettings,
    state: &mut ControllerState,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let (_, right, up) = rot.forward_right_up();
    let (_, move_vec) = input.movement_vector(
        Some(&InputAxis::GamepadLeftStick),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyW)),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyS)),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyA)),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyD)),
    );

    let speed = movement_speed(input, settings);
    let target_move = Vector2::new(move_vec.x as f32, move_vec.y as f32);
    let move_input = smooth_input(state.move_input, target_move, dt, settings.smoothing);
    state.move_input = move_input;

    // Pan in the view plane instead of moving along the view direction.
    pos.0 += right * (move_input.y * speed * dt);
    pos.0 += up * (move_input.x * speed * dt);

    let (gamepad_look, stick) = input.view_vector(Some(&InputAxis::GamepadRightStick));
    let mouse = cursor_grabbed
        .then(|| input.delta_state_any(&InputAxis::MouseMovement))
        .flatten()
        .map(|(_, delta)| Vector2::new(delta.x as f32, delta.y as f32))
        .unwrap_or(Vector2::new(0.0, 0.0));
    let target_look = mouse
        + if gamepad_look {
            Vector2::new(stick.x as f32, stick.y as f32) * STICK_LOOK_SPEED * dt
        } else {
            Vector2::new(0.0, 0.0)
        };
    let target_look = Vector2::new(
        target_look.x * if settings.invert_y { -1.0 } else { 1.0 },
        target_look.y * if settings.invert_x { -1.0 } else { 1.0 },
    );
    let look = smooth_input(state.look_input, target_look, dt, settings.smoothing);
    state.look_input = look;

    rot.rotate_pitch_clamped(
        Rad(look.x * settings.look_sensitivity),
        Rad(settings.pitch_limit),
    );
    rot.rotate_yaw(Rad(-look.y * settings.look_sensitivity));
}

fn movement_speed(input: &InputState, settings: &CameraControlSettings) -> f32 {
    let sprint = input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(
            KeyCode::ShiftLeft,
        )))
        .map(|(_, state)| state)
        .unwrap_or(false)
        || input
            .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(
                KeyCode::ShiftRight,
            )))
            .map(|(_, state)| state)
            .unwrap_or(false);

    settings.move_speed
        * if sprint {
            settings.sprint_multiplier
        } else {
            1.0
        }
}

fn apply_keyboard_mouse_controls(
    dt: f32,
    input: &InputState,
    cursor_grabbed: bool,
    settings: &CameraControlSettings,
    state: &mut ControllerState,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let (forward, right, _up) = rot.forward_right_up();

    // Horizontal movement: the left stick wins when it's deflected,
    // otherwise movement_vector falls back to the WASD buttons.
    // x = forward, y = right (both normalized to [-1, 1]).
    let (_, move_vec) = input.movement_vector(
        Some(&InputAxis::GamepadLeftStick),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyW)),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyS)),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyA)),
        &InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyD)),
    );

    let target_move = Vector2::new(move_vec.x as f32, move_vec.y as f32);
    let move_input = smooth_input(state.move_input, target_move, dt, settings.smoothing);
    state.move_input = move_input;

    let speed = movement_speed(input, settings);
    let mut movement = forward * (move_input.x * speed * dt) + right * (move_input.y * speed * dt);

    // Vertical movement (keyboard only)
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyE)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement.y += speed * dt;
    }
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyQ)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement.y -= speed * dt;
    }
    pos.0 += movement;

    // Mouse rotation — only when cursor is grabbed (hidden).
    let mouse = cursor_grabbed
        .then(|| input.delta_state_any(&InputAxis::MouseMovement))
        .flatten()
        .map(|(_, delta)| Vector2::new(delta.x as f32, delta.y as f32))
        .unwrap_or(Vector2::new(0.0, 0.0));

    // Right-stick look. Signs mirror the mouse-look convention: the stored
    // stick delta is +up / +right, the same frame of reference as
    // normalized mouse deltas. Active regardless of cursor grab state —
    // a gamepad doesn't drive the UI cursor.
    let (gamepad_look, stick) = input.view_vector(Some(&InputAxis::GamepadRightStick));
    let target_look = mouse
        + if gamepad_look {
            Vector2::new(stick.x as f32, stick.y as f32) * STICK_LOOK_SPEED * dt
        } else {
            Vector2::new(0.0, 0.0)
        };
    let target_look = Vector2::new(
        target_look.x * if settings.invert_y { -1.0 } else { 1.0 },
        target_look.y * if settings.invert_x { -1.0 } else { 1.0 },
    );
    let look = smooth_input(state.look_input, target_look, dt, settings.smoothing);
    state.look_input = look;

    rot.rotate_pitch_clamped(
        Rad(look.x * settings.look_sensitivity),
        Rad(settings.pitch_limit),
    );
    rot.rotate_yaw(Rad(-look.y * settings.look_sensitivity));
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Point3;
    use orbital_input::InputEvent;
    use winit::dpi::PhysicalSize;
    use winit::event::DeviceId;

    fn mouse_look(vertical_delta: f64) -> InputState {
        let mut input = InputState::new();
        input.surface_resize(PhysicalSize::new(1000, 800));
        input.handle_event(InputEvent::MouseMovedDelta {
            device_id: DeviceId::dummy(),
            delta: (0.0, vertical_delta),
        });
        input
    }

    #[test]
    fn mouse_pitch_cannot_cross_vertical() {
        for (input, direction) in [
            (mouse_look(-1_000_000.0), 1.0_f32),
            (mouse_look(1_000_000.0), -1.0_f32),
        ] {
            let mut pos = Position(Point3::new(0.0, 0.0, 0.0));
            let mut rot = Rotation::identity();
            let settings = CameraControlSettings::default();
            let mut state = ControllerState::default();

            for _ in 0..10 {
                apply_keyboard_mouse_controls(
                    1.0 / 60.0,
                    &input,
                    true,
                    &settings,
                    &mut state,
                    &mut pos,
                    &mut rot,
                );
            }

            let (forward, _, _) = rot.forward_right_up();
            assert!(
                forward.y * direction <= settings.pitch_limit.sin() + 1e-4,
                "pitch crossed the limit: forward.y={}, direction={direction}",
                forward.y,
            );
        }
    }
}
