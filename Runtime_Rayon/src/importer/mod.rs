//! # Importer Module
//!
//! The importer module handles asset loading and processing for the Orbital engine.
//! It provides a system for importing various asset formats (primarily GLTF) and converting
//! them into engine resources like models and cameras.
//!
//! ## Key Components
//!
//! - **Importer**: Manages the import task queue and runs import operations in parallel
//! - **ImportTask**: Represents different types of import operations that can be queued
//! - **ImportResult**: Contains the results of an import operation (models, cameras, etc.)
//! - **GLTF Import**: Specialized support for GLTF format assets with materials and scenes
//!
//! ## Parallel Processing
//!
//! The importer processes tasks on a rayon thread pool with configurable parallelism,
//! allowing multiple assets to be loaded simultaneously without blocking the main thread.
//! Completed results are collected via a channel.

use std::sync::{mpsc, Mutex};

use crate::{
    importer::gltf::{GltfImport, GltfImportTask, GltfImporter},
    resources::{CameraDescriptor, ModelDescriptor},
};

pub mod gltf;

/// Represents different types of import operations that can be queued.
/// Currently supports GLTF format assets, but designed to support additional formats.
#[derive(Debug)]
pub enum ImportTask {
    Gltf { file_path: String, task: GltfImport },
}

/// Contains the results of an import operation, including any models and cameras
/// that were created during the import process.
#[derive(Default)]
pub struct ImportResult {
    pub models: Vec<ModelDescriptor>,
    pub cameras: Vec<CameraDescriptor>,
}

/// The main importer that manages the import task queue and runs import operations
/// in parallel using a rayon thread pool with configurable parallelism.
pub struct Importer {
    queued_tasks: Vec<ImportTask>,
    result_receiver: Mutex<mpsc::Receiver<ImportResult>>,
    result_sender: mpsc::Sender<ImportResult>,
    pool: rayon::ThreadPool,
}

impl Importer {
    pub fn new(allowed_parallel_tasks: u8) -> Self {
        let num_threads = (allowed_parallel_tasks as usize).max(1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to build rayon thread pool");
        let (sender, receiver) = mpsc::channel();

        Self {
            queued_tasks: Vec::new(),
            result_receiver: Mutex::new(receiver),
            result_sender: sender,
            pool,
        }
    }

    pub fn register_task(&mut self, task: ImportTask) {
        self.queued_tasks.push(task);
    }

    pub fn update(&mut self) -> Vec<ImportResult> {
        let mut results = Vec::new();

        // Drain completed results (non-blocking)
        while let Ok(result) = self.result_receiver.lock().unwrap().try_recv() {
            results.push(result);
        }

        // Spawn new tasks on the rayon thread pool
        while !self.queued_tasks.is_empty() {
            let task_desc = self.queued_tasks.remove(0);
            let sender = self.result_sender.clone();

            self.pool.spawn(move || {
                let result = match task_desc {
                    ImportTask::Gltf { file_path, task } => {
                        let gltf_result = GltfImporter::import(GltfImportTask {
                            file: file_path,
                            import: task,
                        });

                        ImportResult {
                            models: gltf_result.models,
                            cameras: gltf_result.cameras,
                        }
                    }
                };
                let _ = sender.send(result);
            });
        }

        results
    }
}
