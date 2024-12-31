use std::sync::Arc;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue,
};

use crate::{bounding_box::BoundingBox, resources::descriptors::MeshDescriptor};

use super::Vertex;

#[derive(Debug)]
pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    bounding_box: Option<Arc<BoundingBox>>,
    bounding_box_buffer: Option<Buffer>,
}

impl Mesh {
    pub fn from_descriptor(descriptor: &MeshDescriptor, device: &Device, _queue: &Queue) -> Self {
        Self::from_data(
            &descriptor.vertices,
            &descriptor.indices,
            descriptor.bounding_box.clone(),
            device,
        )
    }

    pub fn from_data(
        vertices: &[Vertex],
        indices: &[u32],
        bounding_box: Option<Arc<BoundingBox>>,
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
            label: Some("Mesh Vertex Buffer"),
            contents: &indices
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<u8>>(),
            usage: BufferUsages::INDEX,
        });

        let bounding_box_buffer = bounding_box.clone().map(|x| x.to_binary_data()).map(|x| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Bounding Box Buffer"),
                contents: &x,
                usage: BufferUsages::UNIFORM,
            })
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            bounding_box,
            bounding_box_buffer,
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

    pub fn bounding_box(&self) -> Option<Arc<BoundingBox>> {
        self.bounding_box.clone()
    }

    pub fn bounding_box_buffer(&self) -> Option<&Buffer> {
        self.bounding_box_buffer.as_ref()
    }
}
