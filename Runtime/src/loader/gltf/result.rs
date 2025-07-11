use crate::resources::{CameraDescriptor, ModelDescriptor};
use std::error::Error;

/// Contains the results of a glTF Import.
#[derive(Debug, Default)]
pub struct GltfImportResult {
    pub models: Vec<ModelDescriptor>,
    pub cameras: Vec<CameraDescriptor>,
    pub errors: Vec<Box<dyn Error>>,
}

impl GltfImportResult {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn extend(&mut self, other: Self) {
        self.models.extend(other.models);
        self.cameras.extend(other.cameras);
        self.errors.extend(other.errors);
    }
}
