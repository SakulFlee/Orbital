use std::{cell::RefCell, sync::Arc};

use crate::{
    cache::Cache,
    resources::{
        descriptors::{
            MaterialDescriptor, MeshDescriptor, PipelineDescriptor, ShaderDescriptor,
            TextureDescriptor,
        },
        realizations::{Material, Mesh, Pipeline, Shader, Texture},
    },
};

pub struct CacheState {
    pub mesh_cache: RefCell<Cache<Arc<MeshDescriptor>, Mesh>>,
    pub material_cache: RefCell<Cache<Arc<MaterialDescriptor>, Material>>,
    pub texture_cache: RefCell<Cache<Arc<TextureDescriptor>, Texture>>,
    pub pipeline_cache: RefCell<Cache<Arc<PipelineDescriptor>, Pipeline>>,
    pub shader_cache: RefCell<Cache<Arc<ShaderDescriptor>, Shader>>,
}

impl CacheState {
    pub fn new() -> Self {
        Self {
            mesh_cache: RefCell::new(Cache::new()),
            material_cache: RefCell::new(Cache::new()),
            texture_cache: RefCell::new(Cache::new()),
            pipeline_cache: RefCell::new(Cache::new()),
            shader_cache: RefCell::new(Cache::new()),
        }
    }
}
