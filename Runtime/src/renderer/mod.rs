use std::{cell::RefCell, error::Error, sync::Arc};

use async_std::sync::RwLock;
use cgmath::Vector2;
use hashbrown::HashMap;
use wgpu::{Device, Queue, TextureFormat};

use crate::{
    cache::Cache,
    physics::{ChangeList, ChangeListEntry, ModelChangeListEntry},
    resources::{
        MaterialShader, MaterialShaderDescriptor, Mesh, MeshDescriptor, Model, Texture,
        WorldEnvironment, WorldEnvironmentDescriptor,
    },
};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Renderer {
    surface_format: TextureFormat,
    depth_buffer_texture: Texture,
    world_environment: Option<WorldEnvironment>,
    model_cache: HashMap<String, Arc<RwLock<Model>>>,
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

    pub fn update(&mut self, device: &Device, queue: &Queue, change_list: &ChangeList) {
        for entry in change_list {
            match entry {
                // TODO: World Environment changes
                ChangeListEntry::Model(model_change_list_entry) => todo!(),
                ChangeListEntry::Camera(camera_change_list_entry) => todo!(),
                ChangeListEntry::Clear => todo!(),
            }
        }
    }

    async fn update_model(&mut self, model_change: &ModelChangeListEntry) {
        match model_change {
            ModelChangeListEntry::Spawn(model) => {
                let lock = model.read().await;
                let label = lock.label.clone();
                self.model_cache.insert(label, model.clone());
            }
            ModelChangeListEntry::Despawn(model) => todo!(),
            ModelChangeListEntry::Change(model) => todo!(),
        }
    }
}

// TODO: Caches for Model
// TODO: Chunking
