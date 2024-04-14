use ulid::Ulid;
use wgpu::{Buffer, BufferUsages};

use super::{Resource, Vertex, VertexRaw};
use crate::runtime::Context;

pub struct Mesh {
    ulid: Ulid,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl Mesh {
    pub fn from_vertex_index(context: &Context, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self::from_vertex_index_raw(
            context,
            vertices.iter().map(|x| x.into()).collect(),
            indices,
        )
    }

    pub fn from_vertex_index_raw(
        context: &Context,
        vertices: Vec<VertexRaw>,
        indices: Vec<u32>,
    ) -> Self {
        let ulid = Ulid::new();

        let vertex_buffer = context.make_buffer(
            Some(&format!("Mesh Vertex Buffer#{}", ulid)),
            &vertices,
            BufferUsages::VERTEX,
        );
        let index_buffer = context.make_buffer(
            Some(&format!("Mesh Index Buffer#{}", ulid)),
            &indices,
            BufferUsages::INDEX,
        );

        Self::from_buffer(
            Some(ulid),
            vertex_buffer,
            index_buffer,
            indices.len() as u32,
        )
    }

    pub fn from_buffer(
        ulid: Option<Ulid>,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        index_count: u32,
    ) -> Self {
        Self {
            ulid: ulid.unwrap_or(Ulid::new()),
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

impl Resource for Mesh {
    fn ulid(&self) -> &Ulid {
        &self.ulid
    }

    fn set_ulid(&mut self, ulid: Ulid) {
        self.ulid = ulid;
    }
}
