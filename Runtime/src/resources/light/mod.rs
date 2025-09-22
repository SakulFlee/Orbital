use std::error::Error;
use std::sync::Arc;

use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

use crate::resources::Transform;

mod descriptor;
pub use descriptor::*;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Light {
    descriptor: LightDescriptor,
}

impl Light {
    pub fn from_descriptor(descriptor: &LightDescriptor, _device: &Device, _queue: &Queue) -> Result<Self, Box<dyn Error>> {
        let light = Self {
            descriptor: descriptor.clone(),
        };
        Ok(light)
    }

    pub fn descriptor(&self) -> &LightDescriptor {
        &self.descriptor
    }
}
