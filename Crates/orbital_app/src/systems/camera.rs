//! Camera controller system — WASD movement + mouse look, with automatic
//! touch-screen fallback (virtual joystick + drag-to-look).
//!
//! Reads `DeltaTime` and `InputSnapshot` resources, writes `Position` and
//! `Rotation` components on entities with a camera.

use cgmath::{Rad, Vector3};
use orbital_ecs::Res;
use orbital_ecs_bridge::{DeltaTime, InputSnapshot, Position, Rotation};
use orbital_input::{InputAxis, InputButton, InputState};
use winit::keyboard::{KeyCode, PhysicalKey};

const MOVE_SPEED: f32 = 5.0;
const LOOK_SENSITIVITY: f32 = 1.5;

/// Camera controller system: WASD movement + mouse look.
///
/// When any finger is touching the screen (mobile), it automatically switches
/// to touch controls instead ([`sys_touch_camera_controller`]).
///
/// Reads `Res<DeltaTime>`, `Res<InputSnapshot>`, writes `&mut Position`, `&mut Rotation`.
/// Must be a named function (not inline closure) for IntoSystem macro compatibility.
pub fn sys_camera_controller(
    dt: Res<DeltaTime>,
    input: Res<InputSnapshot>,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    if input.0.has_active_touches() {
        apply_touch_controls(dt.0 as f32, &input.0, pos, rot);
    } else {
        apply_keyboard_mouse_controls(dt.0 as f32, &input.0, pos, rot);
    }
}

/// Touch-screen camera controller: left half = virtual joystick (move),
/// right half = drag to look. No-op when no finger is touching.
///
/// Same signature as [`sys_camera_controller`] so it can be scheduled directly
/// by games that always want touch semantics.
pub fn sys_touch_camera_controller(
    dt: Res<DeltaTime>,
    input: Res<InputSnapshot>,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    apply_touch_controls(dt.0 as f32, &input.0, pos, rot);
}

fn apply_touch_controls(dt: f32, input: &InputState, pos: &mut Position, rot: &mut Rotation) {
    let gesture = input.touch_gesture();
    if !gesture.active {
        return;
    }

    let (forward, right, _up) = rot.forward_right_up();

    let movement = forward * (gesture.move_vector.y as f32 * MOVE_SPEED * dt)
        + right * (gesture.move_vector.x as f32 * MOVE_SPEED * dt);
    pos.0 += movement;

    // Drag-to-look on the right half. `look_delta` is in screen pixels
    // (x = horizontal, y = vertical with positive = down). Signs mirror the
    // mouse-look convention so both input styles feel identical.
    let look = gesture.look_delta;
    rot.rotate_pitch(Rad(-look.y as f32 * LOOK_SENSITIVITY));
    rot.rotate_yaw(Rad(-look.x as f32 * LOOK_SENSITIVITY));
}

fn apply_keyboard_mouse_controls(
    dt: f32,
    input: &InputState,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let (forward, right, _up) = rot.forward_right_up();

    // WASD movement
    let mut movement = Vector3::new(0.0, 0.0, 0.0);
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyW)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement += forward * MOVE_SPEED * dt;
    }
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyS)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement -= forward * MOVE_SPEED * dt;
    }
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyD)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement += right * MOVE_SPEED * dt;
    }
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyA)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement -= right * MOVE_SPEED * dt;
    }
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyE)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement.y += MOVE_SPEED * dt;
    }
    if input
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyQ)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement.y -= MOVE_SPEED * dt;
    }
    pos.0 += movement;

    // Mouse rotation
    // delta.x = mouse Y movement (normalized), delta.y = mouse X movement (normalized)
    if let Some((_, delta)) = input.delta_state_any(&InputAxis::MouseMovement) {
        rot.rotate_pitch(Rad(delta.x as f32 * LOOK_SENSITIVITY)); // mouse Y → pitch (up/down)
        rot.rotate_yaw(Rad(-delta.y as f32 * LOOK_SENSITIVITY)); // mouse X → yaw (negated: right = positive)
    }
}
