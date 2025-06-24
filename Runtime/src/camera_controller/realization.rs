use crate::app::input::{InputAxis, InputState};
use crate::app::AppEvent;
use crate::camera_controller::{
    CameraControllerDescriptor, CameraControllerMouseInputMode, CameraControllerMouseInputType,
    CameraControllerMovementType, CameraControllerRotationType,
};
use crate::element::{
    CameraEvent, Element, ElementRegistration, Event, Message, Variant, WorldEvent,
};
use crate::resources::{CameraTransform, Mode};
use async_trait::async_trait;
use cgmath::{Vector2, Zero};
use log::{debug, warn};
use std::sync::Arc;

#[derive(Debug)]
pub struct CameraController {
    descriptor: CameraControllerDescriptor,
    resolution: Option<Vector2<u32>>,
}

impl CameraController {
    pub fn new(descriptor: CameraControllerDescriptor) -> Self {
        Self {
            descriptor,
            resolution: None,
        }
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

        match &self.descriptor.movement_type {
            CameraControllerMovementType::Input { .. } => {}
            CameraControllerMovementType::Following { .. } => {
                todo!("Following mode isn't supported yet!");
            }
            CameraControllerMovementType::Static => {
                // Nothing here as this means there won't be any automatic movement!
            }
        }

        self.handle_rotation(delta_time, &mut transform, input_state);

        if transform.is_introducing_change() {
            Some(Event::World(WorldEvent::Camera(CameraEvent::Transform(
                transform,
            ))))
        } else {
            None
        }
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
                ignore_pitch_for_forward_movement,
            } => {
                // TODO: Delta input
                // TODO: Button input
                // NOTE: Each input type is handled separately, but should be exclusive.
                //       I.e. Delta over buttons over mouse!
                if let Some(x) = mouse_input {
                    if x.input_type.is_triggering(input_state) {
                        self.apply_mouse_view(
                            transform,
                            delta_time,
                            input_state,
                            *ignore_pitch_for_forward_movement,
                            x.sensitivity,
                        );
                    }
                }
            }
            CameraControllerRotationType::Locked => {
                // Nothing here as this means there won't be any automatic rotation!
            }
        }
    }

    fn apply_mouse_view(
        &self,
        transform: &mut CameraTransform,
        delta_time: f64,
        input_state: &InputState,
        ignore_pitch: bool,
        sensitivity: f32,
    ) {
        let resolution = if let Some(resolution) = self.resolution {
            resolution
        } else {
            warn!("No resolution set! Using default");
            Vector2::new(1, 1)
        };
        if let Some((_, view_vector)) =
            input_state.delta_state_any_normalized(&InputAxis::MouseMovement, resolution)
        {
            if !view_vector.is_zero() {
                if ignore_pitch {
                    transform.pitch = Some(Mode::OffsetViewAlignedWithY(
                        ((view_vector.x * delta_time) as f32) * sensitivity,
                    ));
                    transform.yaw = Some(Mode::OffsetViewAlignedWithY(
                        ((view_vector.y * delta_time) as f32) * sensitivity,
                    ));
                } else {
                    transform.pitch = (Some(Mode::OffsetViewAligned(
                        (view_vector.x * delta_time) as f32 * sensitivity,
                    )));
                    transform.yaw = (Some(Mode::OffsetViewAligned(
                        (view_vector.y * delta_time) as f32 * sensitivity,
                    )));
                }
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
        if let Some(Variant::String(type_string)) = message.content().get("Type") {
            if type_string == "WindowResize" {
                if let Some(Variant::U32(width)) = message.content().get("Width") {
                    if let Some(Variant::U32(height)) = message.content().get("Height") {
                        let resolution = Vector2::new(*width, *height);
                        self.resolution = Some(resolution);

                        return None;
                    }
                }
            }
        }

        #[cfg(debug_assertions)]
        warn!("Received unknown message: {:#?}", message);

        None
    }

    async fn on_update(&mut self, delta_time: f64, input_state: &InputState) -> Option<Vec<Event>> {
        self.update_camera(delta_time, input_state).map(|x| vec![x])
    }
}
