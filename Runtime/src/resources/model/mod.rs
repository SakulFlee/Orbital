use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue, TextureFormat,
};

pub use super::{Mesh, MeshDescriptor};
use crate::{
    cache::{Cache, CacheEntry},
    resources::{Instance, MaterialShader, MaterialShaderDescriptor},
};

mod descriptor;
pub use descriptor::*;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Model {
    mesh: Arc<Mesh>,
    materials: Vec<Arc<MaterialShader>>,
    instance_count: u32,
    instance_buffer: Buffer,
}

impl Model {
    pub fn from_descriptor<'cache>(
        descriptor: &ModelDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
        mesh_cache: &'cache RwLock<Cache<Arc<MeshDescriptor>, Mesh>>,
        material_cache: &'cache RwLock<Cache<Arc<MaterialShaderDescriptor>, MaterialShader>>,
    ) -> Result<Self, Box<dyn Error + 'cache>> {
        // --- Mesh ---
        let mesh = match mesh_cache.write() {
            Ok(mut lock) => lock
                .entry(descriptor.mesh.clone())
                .or_insert(CacheEntry::new(Mesh::from_descriptor(
                    &descriptor.mesh,
                    device,
                    queue,
                )))
                .clone_inner(),
            Err(e) => return Err(Box::new(e)),
        };

        // --- Material ---
        let mut materials = Vec::new();
        for material_descriptor in &descriptor.materials {
            materials.push(match material_cache.write() {
                Ok(mut lock) => lock
                    .entry(material_descriptor.clone())
                    .or_insert(CacheEntry::new(MaterialShader::from_descriptor(
                        material_descriptor,
                        Some(*surface_format),
                        device,
                        queue,
                    )?))
                    .clone_inner(),
                Err(e) => return Err(Box::new(e)),
            });
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
