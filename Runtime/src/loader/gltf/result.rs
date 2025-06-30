use crate::resources::{CameraDescriptor, ModelDescriptor};

/// Contains the results of a glTF Import.
#[derive(Debug)]
pub struct GltfImportResult {
    pub models: Option<Vec<ModelDescriptor>>,
    pub cameras: Option<Vec<CameraDescriptor>>,
}
