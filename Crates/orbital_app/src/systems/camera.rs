//! Camera controller system — WASD movement + mouse look.
//!
//! Reads `DeltaTime` and `InputSnapshot` resources, writes `Position` and
//! `Rotation` components on entities with a camera.

use orbital_ecs::Res;
use orbital_ecs_bridge::{DeltaTime, InputSnapshot, Position, Rotation};
use orbital_input::{InputAxis, InputButton};
use cgmath::{Rad, Vector3};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Camera controller system: WASD movement + mouse look.
///
/// Reads `Res<DeltaTime>`, `Res<InputSnapshot>`, writes `&mut Position`, `&mut Rotation`.
/// Must be a named function (not inline closure) for IntoSystem macro compatibility.
pub fn sys_camera_controller(
    dt: Res<DeltaTime>,
    input: Res<InputSnapshot>,
    pos: &mut Position,
    rot: &mut Rotation,
) {
    let speed = 5.0_f32;
    let sensitivity = 0.003_f32;
    let dt = dt.0 as f32;

    let (forward, right, _up) = rot.forward_right_up();

    // WASD movement
    let mut movement = Vector3::new(0.0, 0.0, 0.0);
    if input.0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyW)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement += forward * speed * dt;
    }
    if input.0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyS)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement -= forward * speed * dt;
    }
    if input.0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyD)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement += right * speed * dt;
    }
    if input.0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyA)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement -= right * speed * dt;
    }
    if input.0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyE)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement.y += speed * dt;
    }
    if input.0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyQ)))
        .map(|(_, s)| s)
        .unwrap_or(false)
    {
        movement.y -= speed * dt;
    }
    pos.0 += movement;

    // Mouse rotation
    if let Some((_, delta)) = input.0.delta_state_any(&InputAxis::MouseMovement) {
        rot.rotate_yaw(Rad(-delta.x as f32 * sensitivity));
        rot.rotate_pitch(Rad(-delta.y as f32 * sensitivity));
    }
}
