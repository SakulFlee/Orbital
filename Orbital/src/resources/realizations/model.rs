use std::sync::Arc;

use instance::Instance;
use material_shader::MaterialShader;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue, TextureFormat,
};

use crate::{
    cache::CacheEntry, cache_state::CacheState, error::Error,
    resources::descriptors::ModelDescriptor,
};

use super::Mesh;

#[derive(Debug)]
pub struct Model {
    mesh: Arc<Mesh>,
    materials: Vec<Arc<MaterialShader>>,
    instance_count: u32,
    instance_buffer: Buffer,
}

impl Model {
    pub fn from_descriptor(
        descriptor: &ModelDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
        app_name: &str,
        with_cache_state: Option<&CacheState>,
    ) -> Result<Self, Error> {
        // --- Mesh ---
        let mesh = if let Some(cache) = with_cache_state {
            cache
                .mesh_cache
                .borrow_mut()
                .entry(descriptor.mesh.clone())
                .or_insert(CacheEntry::new(Mesh::from_descriptor(
                    &descriptor.mesh,
                    device,
                    queue,
                )))
                .clone_inner()
        } else {
            Arc::new(Mesh::from_descriptor(&descriptor.mesh, device, queue))
        };

        // --- Material ---
        let mut materials = Vec::new();
        if let Some(cache) = with_cache_state {
            for material_descriptor in &descriptor.materials {
                materials.push(
                    cache
                        .material_cache
                        .borrow_mut()
                        .entry(material_descriptor.clone())
                        .or_insert(CacheEntry::new(MaterialShader::from_descriptor(
                            &material_descriptor,
                            Some(*surface_format),
                            device,
                            queue,
                        )))
                        .clone_inner(),
                );
            }
        } else {
            for material_descriptor in &descriptor.materials {
                materials.push(Arc::new(MaterialShader::from_descriptor(
                    &material_descriptor,
                    Some(*surface_format),
                    device,
                    queue,
                )));
            }
        }

        // --- Instances ---
        // Take Transform count == Instance count
        let instance_count = descriptor.transforms.len() as u32;

        // Turn Transforms into Instances
        let instances = descriptor
            .transforms
            .iter()
            .map(Instance::from)
            .collect::<Vec<_>>();
        // Turn Instances into buffer data (bytes)
        let instance_buffer_data: Vec<u8> = instances
            .iter()
            .flat_map(|x| x.to_buffer_data())
            .flatten()
            .collect();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Model Instance Buffer"),
            contents: &instance_buffer_data,
            usage: BufferUsages::VERTEX,
        });

        Ok(Self {
            mesh,
            materials,
            instance_count,
            instance_buffer,
        })
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn materials(&self) -> &Vec<Arc<MaterialShader>> {
        &self.materials
    }

    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }

    pub fn instance_buffer(&self) -> &Buffer {
        &self.instance_buffer
    }
}
