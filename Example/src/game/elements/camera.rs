use std::f32::consts::PI;

use orbital::{
    app::{AppChange, InputEvent, WINDOW_HALF_SIZE},
    cgmath::{Point3, Vector3},
    game::{CameraChange, Element, ElementRegistration, Mode, WorldChange},
    gilrs::{Axis, Button},
    log::debug,
    resources::descriptors::CameraDescriptor,
    util::InputHandler,
    winit::{
        dpi::PhysicalPosition,
        keyboard::{KeyCode, PhysicalKey},
    },
};

#[derive(Debug)]
pub struct Camera {
    input_handler: InputHandler,
    is_focused: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub const IDENTIFIER: &'static str = "Main Camera";

    pub const MOVEMENT_SPEED: f32 = 5.0;
    pub const MOUSE_SENSITIVITY: f32 = 0.0075;
    pub const GAMEPAD_SENSITIVITY: f32 = 2.5;

    //--- Keyboard bindings
    pub const KEY_MOVE_FORWARD: PhysicalKey = PhysicalKey::Code(KeyCode::KeyW);
    pub const KEY_MOVE_BACKWARD: PhysicalKey = PhysicalKey::Code(KeyCode::KeyS);
    pub const KEY_MOVE_LEFT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyA);
    pub const KEY_MOVE_RIGHT: PhysicalKey = PhysicalKey::Code(KeyCode::KeyD);
    pub const KEY_MOVE_DOWN: PhysicalKey = PhysicalKey::Code(KeyCode::KeyQ);
    pub const KEY_MOVE_UP: PhysicalKey = PhysicalKey::Code(KeyCode::KeyE);

    pub const KEY_DEBUG: PhysicalKey = PhysicalKey::Code(KeyCode::Space);

    //--- Button bindings
    pub const BUTTON_MOVE_DOWN: Button = Button::DPadDown;
    pub const BUTTON_MOVE_UP: Button = Button::DPadUp;

    //--- Button actions
    pub const ACTION_MOVE_FORWARD: &'static str = "move_forward";
    pub const ACTION_MOVE_BACKWARD: &'static str = "move_backward";
    pub const ACTION_MOVE_LEFT: &'static str = "move_left";
    pub const ACTION_MOVE_RIGHT: &'static str = "move_right";
    pub const ACTION_MOVE_DOWN: &'static str = "move_down";
    pub const ACTION_MOVE_UP: &'static str = "move_up";

    pub const ACTION_DEBUG: &'static str = "debug";

    //--- Axis bindings
    pub const AXIS_MOVE_FORWARD_BACKWARD: Axis = Axis::LeftStickY;
    pub const AXIS_MOVE_LEFT_RIGHT: Axis = Axis::LeftStickX;
    pub const AXIS_LOOK_UP_DOWN: Axis = Axis::RightStickY;
    pub const AXIS_LOOK_LEFT_RIGHT: Axis = Axis::RightStickX;

    //--- Axis actions
    pub const ACTION_MOVE_FORWARD_BACKWARD: &'static str = "move_forward_backward";
    pub const ACTION_MOVE_LEFT_RIGHT: &'static str = "move_left_right";
    pub const ACTION_MOVE_UP_DOWN: &'static str = "move_up_down";
    pub const ACTION_LOOK_LEFT_RIGHT: &'static str = "look_left_right";
    pub const ACTION_LOOK_UP_DOWN: &'static str = "look_up_down";

    pub fn new() -> Self {
        let mut input_handler = InputHandler::new();

        //--- Keyboard bindings
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_FORWARD, Self::ACTION_MOVE_FORWARD);
        input_handler
            .register_keyboard_mapping(Self::KEY_MOVE_BACKWARD, Self::ACTION_MOVE_BACKWARD);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_LEFT, Self::ACTION_MOVE_LEFT);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_RIGHT, Self::ACTION_MOVE_RIGHT);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_DOWN, Self::ACTION_MOVE_DOWN);
        input_handler.register_keyboard_mapping(Self::KEY_MOVE_UP, Self::ACTION_MOVE_UP);

        input_handler.register_keyboard_mapping(Self::KEY_DEBUG, Self::ACTION_DEBUG);

        //--- Button bindings
        input_handler
            .register_gamepad_button_mapping(Self::BUTTON_MOVE_DOWN, Self::ACTION_MOVE_DOWN);
        input_handler.register_gamepad_button_mapping(Self::BUTTON_MOVE_UP, Self::ACTION_MOVE_UP);

        //--- Axis bindings
        input_handler.register_gamepad_axis_mapping(
            Self::AXIS_MOVE_FORWARD_BACKWARD,
            Self::ACTION_MOVE_FORWARD_BACKWARD,
        );
        input_handler.register_gamepad_axis_mapping(
            Self::AXIS_MOVE_LEFT_RIGHT,
            Self::ACTION_MOVE_LEFT_RIGHT,
        );

        input_handler.register_gamepad_axis_mapping(
            Self::AXIS_LOOK_LEFT_RIGHT,
            Self::ACTION_LOOK_LEFT_RIGHT,
        );
        input_handler
            .register_gamepad_axis_mapping(Self::AXIS_LOOK_UP_DOWN, Self::ACTION_LOOK_UP_DOWN);

        Self {
            input_handler,
            is_focused: true,
        }
    }
}

impl Element for Camera {
    fn on_registration(&mut self) -> ElementRegistration {
        ElementRegistration::new(Self::IDENTIFIER)
            .with_initial_world_change(WorldChange::SpawnCameraAndMakeActive(CameraDescriptor {
                identifier: Self::IDENTIFIER.into(),
                position: Point3::new(5.0, 0.0, 0.0),
                yaw: PI,
                ..Default::default()
            }))
            .with_initial_world_change(WorldChange::AppChange(AppChange::ChangeCursorVisible(
                true, // TODO
            )))
    }

    fn on_focus_change(&mut self, focused: bool) {
        self.is_focused = focused;
        debug!("Focus change: {}", focused);
    }

    fn on_input_event(&mut self, input_event: &InputEvent) {
        self.input_handler.handle_event(input_event);
    }

    fn on_update(&mut self, delta_time: f64) -> Option<Vec<WorldChange>> {
        if !self.is_focused {
            return None;
        }

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

        // Calculate camera rotation
        let (is_axis, yaw_change, pitch_change) = self
            .input_handler
            .calculate_view_change_from_axis_and_cursor(
                Self::ACTION_LOOK_LEFT_RIGHT,
                Self::ACTION_LOOK_UP_DOWN,
            );

        // Compile CameraChange
        let change = CameraChange {
            target: Self::IDENTIFIER,
            position: if position.x != 0.0 || position.y != 0.0 || position.z != 0.0 {
                Some(Mode::OffsetViewAligned(position * Self::MOVEMENT_SPEED))
            } else {
                None
            },
            yaw: Some(Mode::Offset(
                yaw_change
                    * if is_axis {
                        Self::GAMEPAD_SENSITIVITY
                    } else {
                        Self::MOUSE_SENSITIVITY
                    },
            )),
            pitch: Some(Mode::Offset(
                pitch_change
                    * if is_axis {
                        Self::GAMEPAD_SENSITIVITY
                    } else {
                        Self::MOUSE_SENSITIVITY
                    },
            )),
        };

        // Send off, if there is a change
        // let cursor_position = PhysicalPosition::<u32>::from(unsafe { WINDOW_HALF_SIZE });
        let cursor_position = PhysicalPosition::<u32>::from((1920 / 2, 1080 / 2));
        let cursor_position_change =
            WorldChange::AppChange(AppChange::ChangeCursorPosition(cursor_position.into()));

        let mut changes = vec![
            WorldChange::AppChange(AppChange::ChangeCursorGrabbed(true)),
            // cursor_position_change,
        ];

        if self.input_handler.is_triggered(Self::ACTION_DEBUG) {
            changes.push(WorldChange::CleanWorld);
        }

        if change.does_change_something() {
            changes.push(WorldChange::UpdateCamera(change));
        }

        Some(changes)
    }
}
