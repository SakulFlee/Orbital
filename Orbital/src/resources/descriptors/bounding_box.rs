use std::{f32, hash::Hash, sync::Arc};

use cgmath::{num_traits::Float, Point3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBoxDescriptor {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl BoundingBoxDescriptor {
    pub fn new(min: Point3<f32>, max: Point3<f32>) -> Self {
        Self { min, max }
    }

    pub fn to_binary_data(&self) -> Vec<u8> {
        [
            // Min
            self.min.x.to_le_bytes(),
            self.min.y.to_le_bytes(),
            self.min.z.to_le_bytes(),
            // Max
            self.max.x.to_le_bytes(),
            self.max.y.to_le_bytes(),
            self.max.z.to_le_bytes(),
            // Buffer alignment to 32b
            [0u8; 4],
            [0u8; 4],
        ]
        .concat()
    }

    pub fn to_binary_data_disabled_frustum_culling() -> Vec<u8> {
        [
            [0u8; 4], [0u8; 4], [0u8; 4], // Min
            [0u8; 4], [0u8; 4], [0u8; 4],
        ]
        .concat()
    }

    pub fn to_vertices(&self) -> Vec<f32> {
        [
            [self.min.x, self.min.y, self.min.z], // Bottom-left-back (0)
            [self.max.x, self.min.y, self.min.z], // Bottom-right-back (1)
            [self.max.x, self.max.y, self.min.z], // Top-right-back (2)
            [self.min.x, self.max.y, self.min.z], // Top-left-back (3)
            [self.min.x, self.min.y, self.max.z], // Bottom-left-front (4)
            [self.max.x, self.min.y, self.max.z], // Bottom-right-front (5)
            [self.max.x, self.max.y, self.max.z], // Top-right-front (6)
            [self.min.x, self.max.y, self.max.z],
        ]
        .concat()
    }

    pub fn to_vertices_data(&self) -> Vec<u8> {
        let x: Vec<[u8; 4]> = self
            .to_vertices()
            .into_iter()
            .map(|x| x.to_le_bytes())
            .collect();

        x.concat()
    }

    pub fn to_indices(&self) -> Vec<u32> {
        vec![
            0, 1, 2, 2, 3, 0, // Front face
            4, 7, 6, 6, 5, 4, // Back face
            0, 3, 7, 7, 4, 0, // Left face
            1, 2, 6, 6, 5, 1, // Right face
            0, 1, 5, 5, 4, 0, // Bottom face
            3, 2, 6, 6, 7, 3, // Top face
        ]
    }

    pub fn to_indices_data(&self) -> Vec<u8> {
        let x: Vec<[u8; 4]> = self
            .to_indices()
            .into_iter()
            .map(|x| x.to_le_bytes())
            .collect();

        x.concat()
    }

    pub fn to_debug_bounding_box_wireframe_buffers(&self, device: &Device) -> (Buffer, Buffer) {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Bounding Box Debug Vertex Buffer"),
            contents: &self.to_vertices_data(),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Bounding Box Debug Index Buffer"),
            contents: &self.to_indices_data(),
            usage: BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }
}

impl Eq for BoundingBoxDescriptor {}

impl Hash for BoundingBoxDescriptor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.min.x.integer_decode().hash(state);
        self.min.y.integer_decode().hash(state);
        self.min.z.integer_decode().hash(state);
        self.max.x.integer_decode().hash(state);
        self.max.y.integer_decode().hash(state);
        self.max.z.integer_decode().hash(state);
    }
}
