use std::{cell::RefCell, error::Error, sync::Arc};

use cache::Cache;
use cgmath::Vector2;
use hashbrown::HashMap;
use mesh::{Mesh, MeshDescriptor};
use model::{MaterialShader, MaterialShaderDescriptor, Model};
use texture::Texture;
use wgpu::{Device, Queue, TextureFormat};
use world_environment::{WorldEnvironment, WorldEnvironmentDescriptor};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Renderer {
    surface_format: TextureFormat,
    depth_buffer_texture: Texture,
    world_environment: Option<WorldEnvironment>,
    model_cache: HashMap<String, Model>,
    with_mesh_cache: RefCell<Cache<Arc<MeshDescriptor>, Mesh>>,
    with_material_cache: RefCell<Cache<Arc<MaterialShaderDescriptor>, MaterialShader>>,
}

impl Renderer {
    pub fn new(
        surface_format: TextureFormat,
        resolution: Vector2<u32>,
        world_environment_descriptor: Option<WorldEnvironmentDescriptor>,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Box<dyn Error>> {
        let depth_buffer_texture = Texture::depth_texture(&resolution, device, queue);

        let world_environment = if let Some(descriptor) = world_environment_descriptor {
            let world_environment = WorldEnvironment::from_descriptor(&descriptor, device, queue)
                .map_err(|e| Box::new(e))?;
            Some(world_environment)
        } else {
            None
        };

        Ok(Self {
            surface_format,
            depth_buffer_texture,
            world_environment,
            model_cache: HashMap::new(),
            with_mesh_cache: RefCell::new(Cache::new()),
            with_material_cache: RefCell::new(Cache::new()),
        })
    }
}
