use std::future::Future;

use crate::{
    element::Event,
    loader::gltf::{GltfImport, GltfImportTask, GltfImporter},
    resources::{CameraDescriptor, ModelDescriptor},
};

pub mod gltf;

pub enum ImportTask {
    Gltf {
        file_path: &str,
        import_task: ImportTask,
    }
}

pub struct ImportResult {
    models: Vec<ModelDescriptor>,
    cameras: Vec<CameraDescriptor>,
}

pub trait ImportTaskExecuter {
    fn import(&self, path: &str) -> ImportResult;
}

pub struct Importer {
    queued_tasks: Vec<ImportTask>,
    running_tasks: Vec<?>, // TODO: Async type?
    allowed_parallel_tasks: u8,
}

impl Importer {
    fn new() -> Self {
        Self {
            queued_tasks: Vec::new(),
            running_tasks: Vec::new(),
            allowed_parallel_tasks: 4,
        }
    }
    
    fn register_task(&mut self, task: ImportTask) {
        // Register tasks to queue
    }

    fn update(&mut self) {
        // Check if any running tasks are ready
        // If so: Extract the results
        
        // Check if less tasks are running than the allowed parallel tasks and if more tasks are queued
        // If so: Start a new task
    }
}

