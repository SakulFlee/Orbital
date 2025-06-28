use orbital::{
    app::{AppRuntime, AppSettings},
    logging::{self, error, info},
    make_main,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

use orbital::app::standard::StandardApp;

mod element;
use element::*;
use orbital::app::input::InputEvent::GamepadButton;
use orbital::app::input::{InputAxis, InputButton};
use orbital::camera_controller::{
    ButtonAxis, CameraController, CameraControllerAxisInputMode, CameraControllerButtonInputMode,
    CameraControllerDescriptor, CameraControllerMouseInputMode, CameraControllerMouseInputType,
    CameraControllerMovementType, CameraControllerRotationType,
};
use orbital::gilrs::Button;
use orbital::winit::keyboard::{KeyCode, PhysicalKey};

pub const NAME: &str = "Orbital-Demo-Project: SkyBox";

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
        Box::new(DebugWorldEnvironment::new()),
    ]);

    match AppRuntime::liftoff(event_loop, app_settings, app) {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

make_main!(entrypoint);
