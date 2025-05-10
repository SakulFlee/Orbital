use async_std::sync::{Arc, RwLock};

use crate::resources::{CameraDescriptor, ModelDescriptor};

#[derive(Debug)]
pub enum ChangeListType {
    Model(Arc<RwLock<ModelDescriptor>>),
    Camera(Arc<RwLock<CameraDescriptor>>),
    // TODO: Light
    All,
}
