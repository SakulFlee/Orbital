//! Light realized GPU type.
//!
//! `Light` implements `orbital_ecs::Component` via the blanket impl
//! (`Debug + Clone + Send + Sync` of its inner `LightDescriptor`),
//! so it can be attached to an entity in the ECS world.

use std::error::Error;

use wgpu::{Device, Queue};

mod descriptor;
pub use descriptor::*;

#[derive(Debug)]
pub struct Light {
    descriptor: LightDescriptor,
}

impl Light {
    pub fn from_descriptor(
        descriptor: &LightDescriptor,
        _device: &Device,
        _queue: &Queue,
    ) -> Result<Self, Box<dyn Error>> {
        let light = Self {
            descriptor: descriptor.clone(),
        };
        Ok(light)
    }

    pub fn descriptor(&self) -> &LightDescriptor {
        &self.descriptor
    }
}
