use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

mod descriptor;
pub use descriptor::*;

#[cfg(test)]
mod tests;

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
