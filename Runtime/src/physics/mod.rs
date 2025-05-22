use log::warn;

use crate::element::{CameraEvent, Event};

mod store;
pub use store::*;

mod event;
pub use event::*;

pub struct Physics {
    model_store: ModelStore,
    camera_store: CameraStore,
}

impl Physics {
    pub fn new() -> Self {
        // TODO: implement a real physics system
        warn!("JUST A DUMMY PLACEHOLDER PHYSICS SYSTEM FOR NOW!");

        Self {
            model_store: ModelStore::new(),
            camera_store: CameraStore::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32, events: Vec<PhysicsEvent>) {
        for event in events {
            // TODO ...
        }
    }
}
