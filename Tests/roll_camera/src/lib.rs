use async_trait::async_trait;
use orbital::app::input::{InputAxis, InputButton};
use orbital::app::standard::StandardApp;
use orbital::app::{AppRuntime, AppSettings};
use orbital::camera_controller::{
    ButtonAxis, CameraController, CameraControllerAxisInputMode, CameraControllerButtonInputMode,
    CameraControllerDescriptor, CameraControllerMouseInputMode, CameraControllerMouseInputType,
    CameraControllerMovementType, CameraControllerRotationType,
};
use orbital::element::{CameraEvent, Element, ElementRegistration, Event, Message, WorldEvent};
use orbital::gilrs::Button;
use orbital::resources::{CameraTransform, Mode};
use orbital::winit::keyboard::{KeyCode, PhysicalKey};
use orbital::{
    logging::{self, error, info},
    make_android_main,
    winit::{error::EventLoopError, event_loop::EventLoop},
};
use std::sync::Arc;

mod elements;
use elements::*;

pub const NAME: &str = "Orbital-Demo-Project: RollCamera";

#[derive(Debug)]
struct RollTestElement {
    /// Roll speed in r/s (radians per second)
    roll_speed: f32,
}

impl RollTestElement {
    fn new() -> Self {
        Self { roll_speed: 2.5 }
    }
}

#[async_trait]
impl Element for RollTestElement {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new("RollTestElement")
    }

    async fn on_message(&mut self, _message: &Arc<Message>) -> Option<Vec<Event>> {
        None
    }

    async fn on_update(
        &mut self,
        delta_time: f64,
        _input_state: &orbital::app::input::InputState,
    ) -> Option<Vec<Event>> {
        Some(vec![Event::World(WorldEvent::Camera(
            CameraEvent::Transform(CameraTransform {
                label: "Default".to_string(),
                position: None,
                pitch: None,
                yaw: None,
                roll: Some(Mode::Offset(self.roll_speed * delta_time as f32)),
            }),
        ))])
    }
}

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = false;
    app_settings.name = NAME.to_string();

    let app = StandardApp::with_initial_elements(vec![
        Box::new(CameraController::new(CameraControllerDescriptor {
            movement_type: CameraControllerMovementType::Input {
                axis: Some(InputAxis::GamepadLeftStick),
                button_axis: Some(vec![ButtonAxis {
                    forward: InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyW)),
                    backward: InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyS)),
                    left: InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyA)),
                    right: InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyD)),
                }]),
                button_up: Some(InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyE))),
                button_down: Some(InputButton::Keyboard(PhysicalKey::Code(KeyCode::KeyQ))),
                speed: 1.0,
                ignore_pitch_for_forward_movement: true,
                axis_dead_zone: 0.1,
            },
            rotation_type: CameraControllerRotationType::Free {
                mouse_input: Some(CameraControllerMouseInputMode {
                    input_type: CameraControllerMouseInputType::Always,
                    sensitivity: 1.0,
                    grab_cursor: true,
                    hide_cursor: true,
                }),
                axis_input: Some(CameraControllerAxisInputMode {
                    axis: vec![InputAxis::GamepadRightStick],
                    sensitivity: 1.0,
                }),
                button_input: Some(CameraControllerButtonInputMode {
                    button_axis: vec![
                        ButtonAxis {
                            forward: InputButton::Keyboard(PhysicalKey::Code(KeyCode::ArrowUp)),
                            backward: InputButton::Keyboard(PhysicalKey::Code(KeyCode::ArrowDown)),
                            left: InputButton::Keyboard(PhysicalKey::Code(KeyCode::ArrowLeft)),
                            right: InputButton::Keyboard(PhysicalKey::Code(KeyCode::ArrowRight)),
                        },
                        ButtonAxis {
                            forward: InputButton::Gamepad(Button::DPadUp),
                            backward: InputButton::Gamepad(Button::DPadDown),
                            left: InputButton::Gamepad(Button::DPadLeft),
                            right: InputButton::Gamepad(Button::DPadRight),
                        },
                    ],
                    sensitivity: 1.0,
                }),
                axis_dead_zone: 0.1,
            },
            camera_descriptor: Default::default(),
        })),
        Box::new(RollTestElement::new()),
        Box::new(WorldEnvironment),
    ]);

    match AppRuntime::liftoff(event_loop, app_settings, app) {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

make_android_main!(entrypoint);
