use std::ops::Range;

use wgpu::{Buffer, BufferUsages, Device};

use crate::engine::{BufferHelper, TMaterial, TMesh, VertexPoint};

#[derive(Debug)]
pub struct StandardMesh {
    name: Option<String>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    instance_range: Range<u32>,
    material: Option<Box<dyn TMaterial>>,
}

impl StandardMesh {
    pub fn from_raw(
        name: Option<&str>,
        device: &Device,
        vertices: Vec<VertexPoint>,
        indices: Vec<u32>,
        instances: Range<u32>,
        material: Option<Box<dyn TMaterial>>,
    ) -> Self {
        let label = name.unwrap_or("Unknown");

        let vertex_buffer = device.make_buffer(
            Some(&format!("{} Vertex Buffer", label)),
            &vertices,
            BufferUsages::VERTEX,
        );
        let index_buffer = device.make_buffer(
            Some(&format!("{} Index Buffer", label)),
            &indices,
            BufferUsages::INDEX,
        );

        Self {
            name: name.map_or(None, |x| Some(x.to_string())),
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            instance_range: instances,
            material,
        }
    }
}

impl TMesh for StandardMesh {
    fn get_vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    fn get_index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    fn get_index_count(&self) -> u32 {
        self.index_count
    }

    fn get_instance_range(&self) -> Range<u32> {
        self.instance_range.clone()
    }

    fn set_instance_range(&mut self, range: Range<u32>) {
        self.instance_range = range;
    }

    fn get_material(&self) -> Option<&Box<dyn TMaterial>> {
        self.material.as_ref()
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
}
