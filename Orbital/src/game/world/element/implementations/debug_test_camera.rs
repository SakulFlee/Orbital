use cgmath::{Point3, Vector3};
use gilrs::{Axis, Button};
use winit::{
    dpi::PhysicalPosition,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    app::{InputEvent, WINDOW_HALF_SIZE},
    game::{CameraChange, Element, ElementRegistration, Mode, WorldChange},
    resources::descriptors::CameraDescriptor,
    util::InputHandler,
};

pub struct DebugTestCamera {
    input_handler: InputHandler,
}

impl DebugTestCamera {
    pub const IDENTIFIER: &'static str = "DEBUG";

    pub const MOVEMENT_SPEED: f32 = 5.0;
    pub const MOUSE_SENSITIVITY: f32 = 0.1;

    // Keyboard bindings
    pub const KEY_MOVE_FORWARD: PhysicalKey = PhysicalKey::Code(KeyCode::KeyW);
    pub const KEY_MOVE_BACKWARD: PhysicalKey = PhysicalKey::Code(KeyCode::KeyS);
    pub const KEY_MOVE_LEFT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyA);
    pub const KEY_MOVE_RIGHT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyD);
    pub const KEY_MOVE_DOWN: PhysicalKey = PhysicalKey::Code(KeyCode::KeyQ);
    pub const KEY_MOVE_UP: PhysicalKey = PhysicalKey::Code(KeyCode::KeyE);

    // Button bindings
    pub const BUTTON_MOVE_DOWN: Button = Button::DPadDown;
    pub const BUTTON_MOVE_UP: Button = Button::DPadUp;

    // Button actions
    pub const ACTION_MOVE_FORWARD: &'static str = "move_forward";
    pub const ACTION_MOVE_BACKWARD: &'static str = "move_backward";
    pub const ACTION_MOVE_LEFT: &'static str = "move_left";
    pub const ACTION_MOVE_RIGHT: &'static str = "move_right";
    pub const ACTION_MOVE_DOWN: &'static str = "move_down";
    pub const ACTION_MOVE_UP: &'static str = "move_up";

    // Axis bindings
    pub const AXIS_FORWARD_BACKWARD: Axis = Axis::LeftStickY;
    pub const AXIS_LEFT_RIGHT: Axis = Axis::LeftStickX;

    // Axis actions
    pub const ACTION_MOVE_FORWARD_BACKWARD: &'static str = "move_forward_backward";
    pub const ACTION_MOVE_LEFT_RIGHT: &'static str = "move_left_right";
    pub const ACTION_MOVE_UP_DOWN: &'static str = "move_up_down";

    pub fn new() -> Self {
        let mut input_handler = InputHandler::new();

        // Keyboard bindings
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_FORWARD, Self::ACTION_MOVE_FORWARD);
        input_handler
            .register_keyboard_mapping(Self::KEY_MOVE_BACKWARD, Self::ACTION_MOVE_BACKWARD);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_LEFT, Self::ACTION_MOVE_LEFT);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_RIGHT, Self::ACTION_MOVE_RIGHT);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_DOWN, Self::ACTION_MOVE_DOWN);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_UP, Self::ACTION_MOVE_UP);

        // Button bindings
        input_handler
            .register_gamepad_button_mapping(Self::BUTTON_MOVE_DOWN, Self::ACTION_MOVE_DOWN);
        input_handler.register_gamepad_button_mapping(Self::BUTTON_MOVE_UP, Self::ACTION_MOVE_UP);

        // Axis bindings
        input_handler.register_gamepad_axis_mapping(
            Self::AXIS_FORWARD_BACKWARD,
            Self::ACTION_MOVE_FORWARD_BACKWARD,
        );
        input_handler
            .register_gamepad_axis_mapping(Self::AXIS_LEFT_RIGHT, Self::ACTION_MOVE_LEFT_RIGHT);

        Self { input_handler }
    }
}

impl Element for DebugTestCamera {
    fn on_registration(&mut self, _ulid: &ulid::Ulid) -> ElementRegistration {
        ElementRegistration {
            tags: Some(vec!["debug test camera".into()]),
            world_changes: Some(vec![
                WorldChange::SpawnCameraAndMakeActive(CameraDescriptor {
                    identifier: Self::IDENTIFIER.into(),
                    position: Point3::new(5.0, 0.0, 0.0),
                    ..Default::default()
                }),
                WorldChange::ChangeCursorVisible(false),
            ]),
            ..Default::default()
        }
    }

    fn on_input_event(&mut self, _delta_time: f64, input_event: &InputEvent) {
        self.input_handler.handle_event(input_event);
    }

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        // Read input axis
        let move_forward_backward = self.input_handler.get_dynamic_axis(
            Self::ACTION_MOVE_FORWARD_BACKWARD,
            Self::ACTION_MOVE_FORWARD,
            Self::ACTION_MOVE_BACKWARD,
        );
        let move_left_right = self.input_handler.get_dynamic_axis(
            Self::ACTION_MOVE_LEFT_RIGHT,
            Self::ACTION_MOVE_RIGHT,
            Self::ACTION_MOVE_LEFT,
        );
        let move_up_down = self.input_handler.get_dynamic_axis(
            Self::ACTION_MOVE_UP_DOWN,
            Self::ACTION_MOVE_UP,
            Self::ACTION_MOVE_DOWN,
        );

        // Calculate camera rotation
        let cursor_position: PhysicalPosition<i32> =
            self.input_handler.get_cursor_position().cast();
        let window_half_size = unsafe { WINDOW_HALF_SIZE };

        let yaw_change = (cursor_position.x - window_half_size.0) as f32 * delta_time as f32;
        let pitch_change = (window_half_size.1 - cursor_position.y) as f32 * delta_time as f32;

        // TODO: Controller Joystick movement

        // Modify position as needed
        let mut position = Vector3::new(0.0, 0.0, 0.0);
        if let Some(axis) = move_forward_backward {
            position.x += axis * delta_time as f32;
        }
        if let Some(axis) = move_left_right {
            position.z += axis * delta_time as f32;
        }
        if let Some(axis) = move_up_down {
            position.y += axis * delta_time as f32;
        }

        // Compile CameraChange
        let change = CameraChange {
            target: Self::IDENTIFIER,
            position: if position.x != 0.0 || position.y != 0.0 || position.z != 0.0 {
                Some(Mode::OffsetViewAligned(position * Self::MOVEMENT_SPEED))
            } else {
                None
            },
            pitch: Some(Mode::Offset(pitch_change * Self::MOUSE_SENSITIVITY)),
            yaw: Some(Mode::Offset(yaw_change * Self::MOUSE_SENSITIVITY)),
        };

        // Send off, if there is a change
        let cursor_position = PhysicalPosition::<u32>::from(unsafe { WINDOW_HALF_SIZE });
        let cursor_position_change = WorldChange::ChangeCursorPosition(cursor_position.into());

        let mut changes = vec![cursor_position_change];

        if change.does_change_something() {
            changes.push(WorldChange::UpdateCamera(change));
        }

        Some(changes)
    }
}
