use std::sync::Arc;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device, Queue, TextureFormat,
};

use crate::{
    cache::{Cache, CacheEntry},
    error::Error,
    resources::descriptors::{
        MaterialDescriptor, MeshDescriptor, ModelDescriptor, PipelineDescriptor, ShaderDescriptor,
        TextureDescriptor,
    },
};

use super::{instance::Instance, Material, Mesh, Pipeline, Shader, Texture};

#[derive(Debug)]
pub struct Model {
    mesh: Arc<Mesh>,
    material: Arc<Material>,
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
        with_mesh_cache: Option<&mut Cache<Arc<MeshDescriptor>, Mesh>>,
        with_material_cache: Option<&mut Cache<Arc<MaterialDescriptor>, Material>>,
        with_texture_cache: Option<&mut Cache<Arc<TextureDescriptor>, Texture>>,
        with_pipeline_cache: Option<&mut Cache<Arc<PipelineDescriptor>, Pipeline>>,
        with_shader_cache: Option<&mut Cache<Arc<ShaderDescriptor>, Shader>>,
    ) -> Result<Self, Error> {
        // --- Mesh ---
        let mesh = if let Some(cache) = with_mesh_cache {
            cache
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
        let material = if let Some(cache) = with_material_cache {
            cache
                .entry(descriptor.material.clone())
                .or_insert(CacheEntry::new(Material::from_descriptor(
                    &descriptor.material,
                    surface_format,
                    device,
                    queue, app_name,
                    with_texture_cache,
                    with_pipeline_cache,
                    with_shader_cache,
                )?))
                .clone_inner()
        } else {
            Arc::new(Material::from_descriptor(
                &descriptor.material,
                surface_format,
                device,
                queue, app_name,
                with_texture_cache,
                with_pipeline_cache,
                with_shader_cache,
            )?)
        };

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
            material,
            instance_count,
            instance_buffer,
        })
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }

    pub fn instance_buffer(&self) -> &Buffer {
        &self.instance_buffer
    }
}
