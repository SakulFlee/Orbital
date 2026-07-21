use wgpu::{
    Buffer, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

pub use crate::Vertex;

mod descriptor;
pub use descriptor::*;

mod cache;
pub use cache::*;

mod bounds;
pub use bounds::*;

#[derive(Debug)]
pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    bounding_sphere: Option<BoundingSphere>,
}

impl Mesh {
    pub fn from_descriptor(descriptor: &MeshDescriptor, device: &Device, _queue: &Queue) -> Self {
        Self::from_data(&descriptor.vertices, &descriptor.indices, device)
    }

    pub fn from_data(vertices: &[Vertex], indices: &[u32], device: &Device) -> Self {
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

        let bounding_sphere = {
            let positions: Vec<_> = vertices.iter().map(|v| v.position).collect();
            BoundingSphere::compute_from_positions(&positions)
        };

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            bounding_sphere,
        }
    }

    pub fn bounding_sphere(&self) -> Option<&BoundingSphere> {
        self.bounding_sphere.as_ref()
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
}
