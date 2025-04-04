use std::{cell::RefCell, sync::Arc};

use material_shader::MaterialShaderDescriptor;

use crate::{
    cache::Cache,
    resources::{
        descriptors::MeshDescriptor,
        realizations::{Material, Mesh, Pipeline, Shader, Texture},
    },
};

pub struct CacheState {
    pub mesh_cache: RefCell<Cache<Arc<MeshDescriptor>, Mesh>>,
    pub material_cache: RefCell<Cache<Arc<MaterialShaderDescriptor>, Material>>,
}

impl CacheState {
    pub fn new() -> Self {
        Self {
            mesh_cache: RefCell::new(Cache::new()),
            material_cache: RefCell::new(Cache::new()),
        }
    }
}
