use std::{
    sync::{Arc, Mutex, OnceLock},
    thread::sleep,
    time::Duration,
};

use log::info;
use winit::window::Window;

use crate::{entity::Entity, gpu_backend::GPUBackend};

#[derive(Debug)]
pub struct Renderer<'a> {
    backend: GPUBackend<'a>,
}

impl Entity for Renderer<'_> {
    fn ulid(&self) -> &ulid::Ulid {
        todo!()
    }

    fn set_ulid(&mut self, ulid: ulid::Ulid) {
        todo!()
    }

    fn event_received(&mut self, identifier: String, event: &dyn std::any::Any) {
        todo!()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        todo!()
    }
}

impl<'a> Renderer<'a> {
    pub(crate) fn new(window: &'a Window) -> Self {
        let backend = GPUBackend::new(Some(&window)).expect("GPU connector failure");

        Self { backend }
    }
}
