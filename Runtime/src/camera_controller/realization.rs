use crate::app::input::{InputAxis, InputButton, InputState};
use crate::app::AppEvent;
use crate::camera_controller::{
    ButtonAxis, CameraControllerDescriptor, CameraControllerMovementType,
    CameraControllerRotationType,
};
use crate::element::{CameraEvent, Element, ElementRegistration, Event, Message, WorldEvent};
use crate::resources::{CameraTransform, Mode};
use async_trait::async_trait;
use cgmath::{Vector2, Vector3, Zero};
use std::sync::Arc;

#[derive(Debug)]
pub struct CameraController {
    descriptor: CameraControllerDescriptor,
}

impl CameraController {
    pub fn new(descriptor: CameraControllerDescriptor) -> Self {
        Self { descriptor }
    }

    pub fn controller_label(&self) -> String {
        format!(
            "StandardCameraController for {}",
            self.descriptor.camera_descriptor.label
        )
    }

    pub fn camera_label(&self) -> String {
        self.descriptor.camera_descriptor.label.clone()
    }

    fn update_camera(&mut self, delta_time: f64, input_state: &InputState) -> Option<Event> {
        let mut transform = CameraTransform {
            label: self.camera_label(),
            position: None,
            pitch: None,
            yaw: None,
        };

        self.handle_movement(delta_time, &mut transform, input_state);
        self.handle_rotation(delta_time, &mut transform, input_state);

        if transform.is_introducing_change() {
            Some(Event::World(WorldEvent::Camera(CameraEvent::Transform(
                transform,
            ))))
        } else {
            None
        }
    }

    fn handle_movement(
        &self,
        delta_time: f64,
        transform: &mut CameraTransform,
        input_state: &InputState,
    ) {
        match &self.descriptor.movement_type {
            CameraControllerMovementType::Input {
                axis,
                button_axis,
                button_up,
                button_down,
                speed,
                ignore_pitch_for_forward_movement,
            } => {
                let mut movement_vector = Vector3::<f64>::zero();

                if let Some(axis) = axis {
                    if let Some(delta_vector) = self.read_delta(axis, input_state) {
                        movement_vector.x += delta_vector.x;
                        movement_vector.z += delta_vector.y;
                    }
                }

                if let Some(button_axis_vec) = button_axis {
                    for button_axis in button_axis_vec {
                        if !movement_vector.is_zero() {
                            break;
                        }
                        let delta_vector = self.read_button_axis(button_axis, input_state);
                        movement_vector.x = delta_vector.x;
                        movement_vector.z = delta_vector.y;
                    }
                }

                if let Some(state) = button_up
                    .map(|x| input_state.button_state_any(&x))
                    .flatten()
                    .map(|(_, state)| state)
                {
                    if state {
                        movement_vector.y = 1.0;
                    }
                }

                if let Some(state) = button_down
                    .map(|x| input_state.button_state_any(&x))
                    .flatten()
                    .map(|(_, state)| state)
                {
                    if state {
                        movement_vector.y = -1.0;
                    }
                }

                let mut output_vector: Vector3<f32> =
                    movement_vector.cast().expect("Cast must succeed!");
                output_vector *= *speed;

                if *ignore_pitch_for_forward_movement {
                    transform.position = Some(Mode::OffsetViewAligned(output_vector));
                } else {
                    transform.position = Some(Mode::OffsetViewAlignedWithY(output_vector));
                }
            }
            CameraControllerMovementType::Following { .. } => {
                todo!("Following mode isn't supported yet!");
            }
            CameraControllerMovementType::Static => {
                // Nothing here as this means there won't be any automatic movement!
            }
        }
    }

    fn read_button_axis(&self, axis: &ButtonAxis, input_state: &InputState) -> Vector2<f64> {
        let forward = input_state
            .button_state_any(&axis.forward)
            .map(|(_, x)| x)
            .unwrap_or(false);
        let backward = input_state
            .button_state_any(&axis.backward)
            .map(|(_, x)| x)
            .unwrap_or(false);
        let left = input_state
            .button_state_any(&axis.left)
            .map(|(_, x)| x)
            .unwrap_or(false);
        let right = input_state
            .button_state_any(&axis.right)
            .map(|(_, x)| x)
            .unwrap_or(false);

        let mut result = Vector2::zero();
        if forward {
            result.x += 1.0;
        }
        if backward {
            result.x -= 1.0;
        }
        if left {
            result.y += 1.0;
        }
        if right {
            result.y -= 1.0;
        }

        result
    }

    fn handle_rotation(
        &self,
        delta_time: f64,
        transform: &mut CameraTransform,
        input_state: &InputState,
    ) {
        match &self.descriptor.rotation_type {
            CameraControllerRotationType::Free {
                mouse_input,
                axis_input,
                button_input,
            } => {
                // TODO: Delta input
                // TODO: Button input
                // NOTE: Each input type is handled separately, but should be exclusive.
                //       I.e. Delta over buttons over mouse!
                if let Some(x) = mouse_input {
                    if x.input_type.is_triggering(input_state) {
                        self.apply_mouse_view(transform, delta_time, input_state, x.sensitivity);
                    }
                }
            }
            CameraControllerRotationType::Locked => {
                // Nothing here as this means there won't be any automatic rotation!
            }
        }
    }

    fn read_button(&self, button: &InputButton, input_state: &InputState) -> Option<bool> {
        input_state.button_state_any(button).map(|(_, x)| x)
    }

    /// Will read a delta state (axis) and return its value if any input got recorded by the [`InputState`].
    /// Upon receiving a delta state (value), if the given state exceeds the standard range (-1.0 to +1.0),
    /// it will be normalized. A value can only be normalized if a resolution has been set prior.
    /// If no resolution has been set beforehand, the value will be dropped, but only if it exceeds
    /// the standard range as defined above.
    fn read_delta(&self, axis: &InputAxis, input_state: &InputState) -> Option<Vector2<f64>> {
        input_state.delta_state_any(axis).map(|(_, x)| x)
    }

    fn apply_mouse_view(
        &self,
        transform: &mut CameraTransform,
        delta_time: f64,
        input_state: &InputState,
        sensitivity: f32,
    ) {
        if let Some(view_vector) = self.read_delta(&InputAxis::MouseMovement, &input_state) {
            if !view_vector.is_zero() {
                transform.pitch = Some(Mode::Offset(view_vector.x as f32 * sensitivity));
                transform.yaw = Some(Mode::Offset(view_vector.y as f32 * sensitivity));
            }
        }
    }
}

#[async_trait]
impl Element for CameraController {
    fn on_registration(&self) -> ElementRegistration {
        let mut registration = ElementRegistration::new(self.camera_label())
            .with_initial_world_changes(vec![
                Event::World(WorldEvent::Camera(CameraEvent::Spawn(
                    self.descriptor.camera_descriptor.clone(),
                ))),
                Event::World(WorldEvent::Camera(CameraEvent::Target(self.camera_label()))),
            ]);

        if let CameraControllerRotationType::Free { mouse_input, .. } =
            &self.descriptor.rotation_type
        {
            if let Some(input_mode) = mouse_input {
                if input_mode.grab_cursor {
                    registration = registration
                        .with_initial_world_change(Event::App(AppEvent::ChangeCursorGrabbed(true)));
                }

                if input_mode.hide_cursor {
                    registration = registration.with_initial_world_change(Event::App(
                        AppEvent::ChangeCursorVisible(false),
                    ));
                }
            }
        }

        registration
    }

    async fn on_message(&mut self, message: &Arc<Message>) -> Option<Vec<Event>> {
        None
    }

    async fn on_update(&mut self, delta_time: f64, input_state: &InputState) -> Option<Vec<Event>> {
        self.update_camera(delta_time, input_state).map(|x| vec![x])
    }
}

// TODO: Reset mouse cursor to center to prevent it from hovering over something else accidentally and escaping the window even though it's grabbed
