use crate::app::input::{InputAxis, InputState};
use crate::app::AppEvent;
use crate::camera_controller::{
    ButtonAxis, CameraControllerAxisInputMode, CameraControllerButtonInputMode,
    CameraControllerDescriptor, CameraControllerMovementType, CameraControllerRotationType,
};
use crate::element::{CameraEvent, Element, ElementRegistration, Event, Message, WorldEvent};
use crate::resources::{CameraTransform, Mode};
use async_trait::async_trait;
use cgmath::num_traits::abs;
use cgmath::{Vector2, Vector3, Zero};
use std::sync::Arc;

#[derive(Debug)]
pub struct CameraController {
    descriptor: CameraControllerDescriptor,
}

impl CameraController {
    const AXIS_NORMALIZATION_TO_MATCH_MOUSE_SENSITIVITY: f32 = 0.01;

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
                axis_dead_zone,
            } => {
                let mut movement_vector = Vector3::<f64>::zero();

                if let Some(axis) = axis {
                    if let Some(delta_vector) = self.read_delta(axis, input_state, *axis_dead_zone)
                    {
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
            result.x += 1.0 * Self::AXIS_NORMALIZATION_TO_MATCH_MOUSE_SENSITIVITY as f64;
        }
        if backward {
            result.x -= 1.0 * Self::AXIS_NORMALIZATION_TO_MATCH_MOUSE_SENSITIVITY as f64;
        }
        if left {
            result.y -= 1.0 * Self::AXIS_NORMALIZATION_TO_MATCH_MOUSE_SENSITIVITY as f64;
        }
        if right {
            result.y += 1.0 * Self::AXIS_NORMALIZATION_TO_MATCH_MOUSE_SENSITIVITY as f64;
        }

        result
    }

    fn apply_delta_to_transform(
        &self,
        delta: &Vector2<f64>,
        transform: &mut CameraTransform,
        sensitivity: f32,
    ) -> bool {
        if !delta.is_zero() {
            transform.pitch = Some(Mode::Offset(delta.x as f32 * sensitivity));
            transform.yaw = Some(Mode::Offset(delta.y as f32 * sensitivity));

            // Early return on the first non-zero existing axis delta
            return true;
        }

        false
    }

    /// Returns `true` if a delta axis with a non-zero value got found and applied to the [`CameraTransform`].
    /// Returns `false` if no delta axis have been found, or, all values returned are zero.
    fn apply_delta_axis_rotation(
        &self,
        mode: &CameraControllerAxisInputMode,
        transform: &mut CameraTransform,
        input_state: &InputState,
        axis_dead_zone: f64,
    ) -> bool {
        for axis in &mode.axis {
            if let Some(delta) = self.read_delta(axis, input_state, axis_dead_zone) {
                return self.apply_delta_to_transform(
                    &delta,
                    transform,
                    mode.sensitivity * Self::AXIS_NORMALIZATION_TO_MATCH_MOUSE_SENSITIVITY,
                );
            }
        }

        false
    }

    fn handle_rotation(
        &self,
        delta_time: f64,
        transform: &mut CameraTransform,
        input_state: &InputState,
    ) {
        match &self.descriptor.rotation_type {
            CameraControllerRotationType::Free {
                axis_input,
                button_input,
                mouse_input,
                axis_dead_zone,
            } => {
                // Delta inputs (gamepad) first
                if axis_input
                    .as_ref()
                    .map(|x| {
                        self.apply_delta_axis_rotation(x, transform, input_state, *axis_dead_zone)
                    })
                    .unwrap_or(false)
                {
                    return;
                }

                // Button inputs next
                if button_input
                    .as_ref()
                    .map(|x| self.apply_button_axis_rotation(x, transform, input_state))
                    .unwrap_or(false)
                {
                    return;
                }

                // Lastly, mouse inputs
                if let Some(x) = mouse_input {
                    x.input_type.is_triggering(input_state).then(|| {
                        self.apply_mouse_view(
                            transform,
                            delta_time,
                            input_state,
                            x.sensitivity,
                            0.0,
                        )
                    });
                }
            }
            CameraControllerRotationType::Locked => {
                // Nothing here as this means there won't be any automatic rotation!
            }
        }
    }

    /// Will read a delta state (axis) and return its value if any input got recorded by the [`InputState`].
    /// Upon receiving a delta state (value), if the given state exceeds the standard range (-1.0 to +1.0),
    /// it will be normalized. A value can only be normalized if a resolution has been set prior.
    /// If no resolution has been set beforehand, the value will be dropped, but only if it exceeds
    /// the standard range as defined above.
    fn read_delta(
        &self,
        axis: &InputAxis,
        input_state: &InputState,
        dead_zone: f64,
    ) -> Option<Vector2<f64>> {
        input_state
            .delta_state_any(axis)
            .and_then(|(_, d)| {
                if abs(d.x) > dead_zone || abs(d.y) > dead_zone {
                    Some(d)
                } else {
                    None
                }
            })
    }

    /// Returns `true` if mouse movement was detected and got applied.
    /// Returns `false` otherwise.
    fn apply_mouse_view(
        &self,
        transform: &mut CameraTransform,
        delta_time: f64,
        input_state: &InputState,
        sensitivity: f32,
        axis_dead_zone: f64,
    ) -> bool {
        if let Some(delta) =
            self.read_delta(&InputAxis::MouseMovement, input_state, axis_dead_zone)
        {
            return self.apply_delta_to_transform(&delta, transform, sensitivity);
        }

        false
    }

    fn apply_button_axis_rotation(
        &self,
        mode: &CameraControllerButtonInputMode,
        transform: &mut CameraTransform,
        input_state: &InputState,
    ) -> bool {
        if let Some(button_axis) = mode.button_axis.first() {
            let delta = self.read_button_axis(button_axis, input_state);
            return self.apply_delta_to_transform(&delta, transform, mode.sensitivity);
        }

        false
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
        // Prevents default message log spam
        None
    }

    async fn on_update(&mut self, delta_time: f64, input_state: &InputState) -> Option<Vec<Event>> {
        self.update_camera(delta_time, input_state).map(|x| vec![x])
    }
}
