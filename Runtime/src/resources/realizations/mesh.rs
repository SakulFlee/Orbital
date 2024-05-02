use wgpu::{Buffer, BufferUsages};

use crate::{
    resources::{descriptors::MeshDescriptor, uniforms::VertexUniform},
    runtime::Context,
};

pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl Mesh {
    pub fn from_descriptor(descriptor: MeshDescriptor, context: &Context) -> Self {
        let vertex_data: Vec<VertexUniform> =
            descriptor.vertices.iter().map(|x| x.into()).collect();

        let vertex_buffer = context.make_buffer(
            Some(&format!("Mesh Vertex Buffer")),
            &vertex_data,
            BufferUsages::VERTEX,
        );
        let index_buffer = context.make_buffer(
            Some(&format!("Mesh Index Buffer")),
            &descriptor.indices,
            BufferUsages::INDEX,
        );

        Self::from_buffer(vertex_buffer, index_buffer, descriptor.indices.len() as u32)
    }

    pub fn from_buffer(vertex_buffer: Buffer, index_buffer: Buffer, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
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
}
