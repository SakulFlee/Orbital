use crate::{
    importer::gltf::{GltfImport, GltfImportTask, GltfImporter},
    resources::{CameraDescriptor, ModelDescriptor},
};
use async_std::task;
use futures::stream::{FuturesUnordered, StreamExt};

pub mod gltf;

#[derive(Debug)]
pub enum ImportTask {
    Gltf { file_path: String, task: GltfImport },
}

#[derive(Default)]
pub struct ImportResult {
    pub models: Vec<ModelDescriptor>,
    pub cameras: Vec<CameraDescriptor>,
}

pub struct Importer {
    queued_tasks: Vec<ImportTask>,
    running_tasks: FuturesUnordered<task::JoinHandle<ImportResult>>,
    allowed_parallel_tasks: u8,
}

impl Importer {
    pub fn new(allowed_parallel_tasks: u8) -> Self {
        Self {
            queued_tasks: Vec::new(),
            running_tasks: FuturesUnordered::new(),
            allowed_parallel_tasks,
        }
    }

    pub fn register_task(&mut self, task: ImportTask) {
        self.queued_tasks.push(task);
    }

    pub async fn update(&mut self) -> Vec<ImportResult> {
        let mut results = Vec::new();

        // Poll the set of running tasks to drain any that have completed.
        // This is non-blocking and will only process futures that are already ready.
        while let Some(result) = self.running_tasks.next().await {
            results.push(result);
        }

        // Check if we can start new tasks.
        while self.running_tasks.len() < self.allowed_parallel_tasks as usize
            && !self.queued_tasks.is_empty()
        {
            let task_desc = self.queued_tasks.remove(0);

            let handle = task::spawn(async move {
                match task_desc {
                    ImportTask::Gltf { file_path, task } => {
                        let gltf_result = GltfImporter::import(GltfImportTask {
                            file: file_path,
                            import: task,
                        })
                        .await;

                        ImportResult {
                            models: gltf_result.models,
                            cameras: gltf_result.cameras,
                        }
                    }
                }
            });

            self.running_tasks.push(handle);
        }

        results
    }
}
