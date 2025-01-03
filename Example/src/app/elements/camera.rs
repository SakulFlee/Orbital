use std::f32::consts::PI;

use orbital::{
    app::AppChange,
    async_trait::async_trait,
    cgmath::{Point3, Vector3},
    gilrs::Button,
    input::{InputAxis, InputButton, InputState},
    resources::descriptors::CameraDescriptor,
    winit::keyboard::{KeyCode, PhysicalKey},
    world::{CameraChange, Element, ElementRegistration, Message, Mode, WorldChange},
};

#[derive(Debug)]
pub struct Camera {}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub const IDENTIFIER: &'static str = "Main Camera";

    pub const MOVEMENT_SPEED: f64 = 5.0;
    pub const SPRINT_MULTIPLIER: f64 = 5.0;

    pub const MOUSE_SENSITIVITY: f64 = 2.0;
    pub const GAMEPAD_MOVEMENT_SENSITIVITY: f64 = 2.5;
    pub const GAMEPAD_VIEW_SENSITIVITY: f64 = 2.5;

    pub const GAMEPAD_MOVEMENT_AXIS: InputAxis = InputAxis::GamepadLeftStick;
    pub const GAMEPAD_VIEW_AXIS: InputAxis = InputAxis::GamepadRightStick;
    pub const GAMEPAD_SPRINT_BUTTON: InputButton = InputButton::Gamepad(Button::East);

    pub const KEYBOARD_MOVEMENT_FORWARD: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyW));
    pub const KEYBOARD_MOVEMENT_BACKWARD: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyS));
    pub const KEYBOARD_MOVEMENT_LEFT: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyA));
    pub const KEYBOARD_MOVEMENT_RIGHT: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyD));
    pub const KEYBOARD_MOVEMENT_SPRINT: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::ShiftLeft));

    pub const KEYBOARD_DEBUG: InputButton =
        InputButton::Keyboard(PhysicalKey::Code(KeyCode::Space));

    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Element for Camera {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new(Self::IDENTIFIER)
            .with_initial_world_change(WorldChange::SpawnCameraAndMakeActive(CameraDescriptor {
                label: Self::IDENTIFIER.into(),
                position: Point3::new(5.0, 0.0, 0.0),
                yaw: PI,
                ..Default::default()
            }))
            .with_initial_world_change(WorldChange::AppChange(AppChange::ChangeCursorVisible(
                false,
            )))
            .with_initial_world_change(WorldChange::AppChange(AppChange::ChangeCursorGrabbed(true)))
    }

    async fn on_update(
        &mut self,
        delta_time: f64,
        input_state: &InputState,
        _messages: Option<Vec<Message>>,
    ) -> Option<Vec<WorldChange>> {
        // Calculate movement vector
        let (movement_vector_is_gamepad, mut movement_vector) = input_state.movement_vector(
            Some(&Self::GAMEPAD_MOVEMENT_AXIS),
            &Self::KEYBOARD_MOVEMENT_FORWARD,
            &Self::KEYBOARD_MOVEMENT_BACKWARD,
            &Self::KEYBOARD_MOVEMENT_LEFT,
            &Self::KEYBOARD_MOVEMENT_RIGHT,
        );
        movement_vector *= delta_time;

        // Check for sprint
        movement_vector *= Self::MOVEMENT_SPEED
            * if input_state
                .button_state_many(&[
                    &Self::KEYBOARD_MOVEMENT_SPRINT,
                    &Self::GAMEPAD_SPRINT_BUTTON,
                ])
                .iter()
                .any(|(_, (_, pressed))| *pressed)
            {
                Self::SPRINT_MULTIPLIER
            } else {
                1.0
            };

        if movement_vector_is_gamepad {
            movement_vector *= Self::GAMEPAD_MOVEMENT_SENSITIVITY;
        }

        // Calculate view vector
        let (view_vector_is_gamepad, mut view_vector) =
            input_state.view_vector(Some(&Self::GAMEPAD_VIEW_AXIS));
        view_vector *= delta_time;

        view_vector *= if view_vector_is_gamepad {
            Self::GAMEPAD_VIEW_SENSITIVITY
        } else {
            Self::MOUSE_SENSITIVITY
        };

        // Make camera change
        let camera_change = CameraChange {
            target: Self::IDENTIFIER,
            // Change to Mode::OffsetViewAligned to enter "view aligned" movement.
            // Change to Mode::OffsetViewAlignedWithY to enter "free cam" movement.
            position: Some(Mode::OffsetViewAlignedWithY(Vector3::new(
                movement_vector.x as f32,
                0.0,
                movement_vector.y as f32,
            ))),
            pitch: Some(Mode::Offset(view_vector.x as f32)),
            yaw: Some(Mode::Offset(view_vector.y as f32)),
        };

        let mut changes = Vec::new();

        // Only submit a camera change if there is something to update
        if camera_change.does_change_something() {
            changes.push(WorldChange::UpdateCamera(camera_change));
        }

        // Only trigger debug action if the button is pressed
        if let Some((_, pressed)) = input_state.button_state_any(&Self::KEYBOARD_DEBUG) {
            if pressed {
                changes.push(WorldChange::CleanWorld);
            }
        }

        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }
}
