use std::sync::Arc;

use async_std::sync::RwLock;

use crate::resources::ModelDescriptor;

#[derive(Debug)]
pub enum ModelChangeListEntry {
    Spawn(Arc<RwLock<ModelDescriptor>>),
    Despawn(Arc<RwLock<ModelDescriptor>>),
    Change(Arc<RwLock<ModelDescriptor>>),
}
