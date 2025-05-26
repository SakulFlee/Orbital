use std::sync::Arc;

use async_std::sync::RwLock;

use crate::resources::CameraDescriptor;

#[derive(Debug)]
pub enum CameraChangeListEntry {
    Spawn(Arc<RwLock<CameraDescriptor>>),
    Despawn(Arc<RwLock<CameraDescriptor>>),
    Change(Arc<RwLock<CameraDescriptor>>),
    Target(Arc<RwLock<CameraDescriptor>>),
}
