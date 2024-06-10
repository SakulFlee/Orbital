use log::warn;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue,
};

use crate::{error::Error, resources::descriptors::MeshDescriptor};

use super::Vertex;

pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
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
                .map(|x| x.to_bytes())
                .flatten()
                .collect::<Vec<u8>>(),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: &indices
                .iter()
                .map(|x| x.to_le_bytes())
                .flatten()
                .collect::<Vec<u8>>(),
            usage: BufferUsages::INDEX,
        });

        Self::from_buffer(vertex_buffer, index_buffer, indices.len() as u32)
    }

    pub fn from_buffer(vertex_buffer: Buffer, index_buffer: Buffer, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }

    #[cfg(feature = "gltf")]
    pub fn from_gltf(gltf_model: &easy_gltf::Model, device: &Device) -> Result<Self, Error> {
        let vertices = gltf_model
            .vertices()
            .iter()
            .map(|vertex| Vertex {
                position_coordinates: vertex.position.into(),
                texture_coordinates: vertex.tex_coords.into(),
            })
            .collect::<Vec<Vertex>>();
        let indices = match gltf_model.indices() {
            Some(i) => i,
            None => {
                warn!("Trying to realize model from glTF without indices!");
                return Err(Error::NoIndices);
            }
        };

        Ok(Self::from_data(&vertices, indices, device))
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
