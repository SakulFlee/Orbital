use orbital_resources::{CameraDescriptor, LightDescriptor, ModelDescriptor};
use std::error::Error;

#[derive(Debug, Default)]
pub struct GltfImportResult {
    pub models: Vec<ModelDescriptor>,
    pub cameras: Vec<CameraDescriptor>,
    pub lights: Vec<LightDescriptor>,
    pub errors: Vec<Box<dyn Error>>,
}

impl GltfImportResult {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn extend(&mut self, other: Self) {
        self.models.extend(other.models);
        self.cameras.extend(other.cameras);
        self.lights.extend(other.lights);
        self.errors.extend(other.errors);
    }
}
