use std::{ops::Bound, sync::Arc};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

use crate::resources::descriptors::BoundingBoxDescriptor;

#[derive(Debug)]
pub struct BoundingBox {
    buffer: Buffer,
}

impl BoundingBox {
    pub fn new(descriptor: &BoundingBoxDescriptor, device: &Device) -> Self {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Bounding Box Buffer"),
            contents: &descriptor.to_binary_data(),
            usage: BufferUsages::UNIFORM,
        });

        Self { buffer }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}
