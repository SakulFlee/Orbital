use std::error::Error;

use log::warn;
use wgpu::{Device, Queue, TextureFormat};

use crate::{
    element::EnvironmentEvent,
    resources::{WorldEnvironment, WorldEnvironmentDescriptor},
};

#[derive(Debug, Default)]
pub struct EnvironmentStore {
    world_environment: Option<WorldEnvironment>,
    queued_descriptor: Option<WorldEnvironmentDescriptor>,
}

impl EnvironmentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_change(&mut self, descriptor: WorldEnvironmentDescriptor) {
        if self.queued_descriptor.is_some() {
            warn!("A WorldEnvironment change has already been queued and will be replaced! Old: {:?}, New: {:?}", &self.queued_descriptor.as_ref().unwrap(), descriptor);
        }

        self.queued_descriptor = Some(descriptor);
    }

    pub fn realize_and_cache(
        &mut self,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<(), Box<dyn Error>> {
        let descriptor = match self.queued_descriptor.take() {
            Some(descriptor) => descriptor,
            None => return Ok(()),
        };

        self.world_environment = Some(WorldEnvironment::from_descriptor(
            &descriptor,
            Some(*surface_format),
            device,
            queue,
        )?);

        Ok(())
    }

    pub fn world_environment(&self) -> Option<&WorldEnvironment> {
        self.world_environment.as_ref()
    }

    pub fn clear(&mut self) {
        self.world_environment = None;
    }

    pub fn handle_event(&mut self, environment_event: EnvironmentEvent) {
        match environment_event {
            EnvironmentEvent::Change {
                descriptor,
                enable_ibl, // TODO
            } => {
                self.queue_change(descriptor);
            }
        }
    }
}
