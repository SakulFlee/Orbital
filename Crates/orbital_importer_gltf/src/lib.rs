pub mod gltf;

use std::sync::{Mutex, mpsc};

use orbital_resources::{CameraDescriptor, ModelDescriptor};

pub use gltf::{GltfImport, GltfImportTask, GltfImporter};

#[derive(Debug)]
pub enum ImportTask {
    Gltf { file_path: String, task: GltfImport },
}

#[derive(Debug, Default)]
pub struct ImportResult {
    pub models: Vec<ModelDescriptor>,
    pub cameras: Vec<CameraDescriptor>,
}

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

        while let Ok(result) = self.result_receiver.lock().unwrap().try_recv() {
            results.push(result);
        }

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
                let _ = sender.send(result).map_err(|_| {
                    log::warn!("Failed to send import result: receiver dropped");
                });
            });
        }

        results
    }
}
