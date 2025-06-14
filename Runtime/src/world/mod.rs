mod store;
use std::time::{Duration, Instant};

pub use store::*;
use wgpu::Device;

use crate::element::WorldEvent;

pub struct World {
    model_store: ModelStore,
    camera_store: CameraStore,
    environment_store: EnvironmentStore,
    last_cleanup: Instant,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            model_store: ModelStore::new(),
            camera_store: CameraStore::new(),
            environment_store: EnvironmentStore::new(),
            last_cleanup: Instant::now(),
        }
    }

    pub fn model_store(&mut self) -> &mut ModelStore {
        &mut self.model_store
    }

    pub fn camera_store(&mut self) -> &mut CameraStore {
        &mut self.camera_store
    }

    pub fn environment_store(&mut self) -> &mut EnvironmentStore {
        &mut self.environment_store
    }

    pub fn update(&mut self, world_events: Vec<WorldEvent>) {
        for world_event in world_events {
            self.process_event(world_event);
        }

        // Needs to be at most the same as the cache timeout time!
        // Otherwise, cache cleanup will never be efficient.
        if self.last_cleanup.elapsed() >= Duration::from_secs(5) {
            self.model_store.cleanup();
            self.camera_store.cleanup();

            self.last_cleanup = Instant::now();
        }
    }

    pub fn process_event(&mut self, event: WorldEvent) {
        match event {
            WorldEvent::Model(model_event) => self.model_store.handle_event(model_event),
            WorldEvent::Camera(camera_event) => self.camera_store.handle_event(camera_event),
            WorldEvent::Environment(environment_event) => {
                self.environment_store.handle_event(environment_event);
            }
            WorldEvent::Clear => {
                self.model_store.clear();
                self.camera_store.clear();
                self.environment_store.clear();
            }
        }
    }

    pub fn prepare_render(&mut self, device: &Device) {
        self.model_store.process_bounding_boxes(device);
    }
}
