use bounding_box::BoundingBox;
use wgpu::{
    Buffer, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

pub use vertex::Vertex;

mod descriptor;
pub use descriptor::*;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    bounding_box: BoundingBox,
}

impl Mesh {
    pub fn from_descriptor(descriptor: &MeshDescriptor, device: &Device, _queue: &Queue) -> Self {
        Self::from_data(
            &descriptor.vertices,
            &descriptor.indices,
            BoundingBox::new(&descriptor.bounding_box, device),
            device,
        )
    }

    pub fn from_data(
        vertices: &[Vertex],
        indices: &[u32],
        bounding_box: BoundingBox,
        device: &Device,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: &vertices
                .iter()
                .flat_map(|x| x.to_bytes())
                .collect::<Vec<u8>>(),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: &indices
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<u8>>(),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            bounding_box,
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }
}
