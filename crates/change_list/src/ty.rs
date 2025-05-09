use async_std::sync::{Arc, RwLock};

use camera::CameraDescriptor;
use model::ModelDescriptor;

#[derive(Debug)]
pub enum ChangeListType {
    Model(Arc<RwLock<ModelDescriptor>>),
    Camera(Arc<RwLock<CameraDescriptor>>),
    // TODO: Light
    All,
}
