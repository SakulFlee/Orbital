use cgmath::{Quaternion, Vector3, Zero};
use wgpu::{Buffer, BufferUsages, Device, Queue};

use crate::engine::{
    BufferHelper, EngineResult, StandardInstance, StandardMaterial, TInstance, TMaterial, TMesh,
    VertexPoint,
};

#[derive(Debug)]
pub struct StandardMesh {
    name: Option<String>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
    instances: Vec<StandardInstance>,
    instance_buffer: Buffer,
    material: Box<dyn TMaterial>,
}

impl StandardMesh {
    pub const MISSING_TEXTURE: &str = "missing_texture.png";

    pub fn from_raw_single(
        name: Option<&str>,
        device: &Device,
        queue: &Queue,
        vertices: Vec<VertexPoint>,
        indices: Vec<u32>,
        material: Option<Box<dyn TMaterial>>,
    ) -> EngineResult<Self> {
        Self::from_raw(
            name,
            device,
            queue,
            vertices,
            indices,
            vec![StandardInstance::new(
                Vector3::zero(),
                Quaternion {
                    v: Vector3::zero(),
                    s: 0.0,
                },
            )],
            material,
        )
    }

    pub fn from_raw(
        name: Option<&str>,
        device: &Device,
        queue: &Queue,
        vertices: Vec<VertexPoint>,
        indices: Vec<u32>,
        instances: Vec<StandardInstance>,
        material: Option<Box<dyn TMaterial>>,
    ) -> EngineResult<Self> {
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

        let instance_uniform = instances
            .iter()
            .map(|x| x.to_instance_uniform())
            .collect::<Vec<_>>();
        let instance_buffer = device.make_buffer(
            Some(&format!("{} Instance Buffer", label)),
            &instance_uniform,
            BufferUsages::VERTEX,
        );

        let material = match material {
            Some(material) => material,
            None => Box::new(StandardMaterial::from_texture(
                device,
                queue,
                Self::MISSING_TEXTURE,
            )?),
        };

        Ok(Self {
            name: name.map(|x| x.to_string()),
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            instances,
            instance_buffer,
            material,
        })
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

    fn get_instances(&mut self) -> &mut Vec<StandardInstance> {
        &mut self.instances
    }

    fn get_instance_count(&self) -> u32 {
        self.instances.len() as u32
    }

    fn get_instance_buffer(&self) -> &Buffer {
        &self.instance_buffer
    }

    fn get_material(&self) -> &Box<dyn TMaterial> {
        &self.material
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
}
