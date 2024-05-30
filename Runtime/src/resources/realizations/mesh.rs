use gltf::{
    accessor::Iter,
    mesh::util::{ReadIndices, ReadTexCoords},
};
use wgpu::{Buffer, BufferUsages, Device, Queue};

use crate::{
    resources::{descriptors::MeshDescriptor, uniforms::VertexUniform},
    util::BufferUtil,
};

pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl Mesh {
    pub fn from_descriptor(descriptor: &MeshDescriptor, device: &Device, _queue: &Queue) -> Self {
        let vertex_data: Vec<VertexUniform> =
            descriptor.vertices.iter().map(|x| x.into()).collect();

        Self::from_data(&vertex_data, &descriptor.indices, device)
    }

    pub fn from_data(vertices: &[VertexUniform], indices: &[u32], device: &Device) -> Self {
        let vertex_buffer =
            device.make_buffer(Some("Mesh Vertex Buffer"), vertices, BufferUsages::VERTEX);
        let index_buffer =
            device.make_buffer(Some("Mesh Index Buffer"), indices, BufferUsages::INDEX);

        Self::from_buffer(vertex_buffer, index_buffer, indices.len() as u32)
    }

    pub fn from_buffer(vertex_buffer: Buffer, index_buffer: Buffer, index_count: u32) -> Self {
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }

    pub fn from_gltf(
        mesh: crate::gltf::Mesh,
        buffers: Vec<gltf::buffer::Data>,
        device: &Device,
    ) -> Self {
        let mut position_coordinates = Vec::<[f32; 3]>::new();
        let mut texture_coordinates = Vec::<[f32; 2]>::new();
        let mut indices = Vec::<u32>::new();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(position) = reader.read_positions() {
                position.for_each(|x| position_coordinates.push(x));
            }

            // Note: Only the first set of texture coordinates is read!
            if let Some(ReadTexCoords::F32(Iter::Standard(texture_coordinate))) =
                reader.read_tex_coords(0)
            {
                texture_coordinate.for_each(|x| texture_coordinates.push(x));
            }

            if let Some(ReadIndices::U32(Iter::Standard(index))) = reader.read_indices() {
                index.for_each(|x| indices.push(x));
            }
        }

        // Make vertices
        let vertices = position_coordinates
            .iter()
            .zip(texture_coordinates.iter())
            .map(
                |(positional_coordinate, texture_coordinate)| VertexUniform {
                    positional_coordinates: *positional_coordinate,
                    texture_coordinates: *texture_coordinate,
                },
            )
            .collect::<Vec<VertexUniform>>();

        // Make mesh
        Self::from_data(&vertices, &indices, device)
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
