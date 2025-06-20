use crate::app::input::InputState;
use crate::camera_controller::CameraControllerDescriptor;
use crate::element::{CameraEvent, Element, ElementRegistration, Event, Message, WorldEvent};
use async_trait::async_trait;

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
}

#[async_trait]
impl Element for CameraController {
    fn on_registration(&self) -> ElementRegistration {
        ElementRegistration::new(self.camera_label()).with_initial_world_changes(vec![
            Event::World(WorldEvent::Camera(CameraEvent::Spawn(
                self.descriptor.camera_descriptor.clone(),
            ))),
            Event::World(WorldEvent::Camera(CameraEvent::Target(self.camera_label()))),
        ])
    }

    async fn on_update(
        &mut self,
        _delta_time: f64,
        _input_state: &InputState,
        _messages: Option<Vec<Message>>,
    ) -> Option<Vec<Event>> {
        None
    }
}
