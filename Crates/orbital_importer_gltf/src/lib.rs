pub mod gltf;

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, mpsc};

use orbital_camera::CameraDescriptor;
use orbital_light::LightDescriptor;
use orbital_model::ModelDescriptor;

pub use gltf::{GltfImport, GltfImportTask, GltfImporter};

#[derive(Debug)]
pub enum ImportTask {
    Gltf { file_path: String, task: GltfImport },
}

#[derive(Debug, Default)]
pub struct ImportResult {
    pub models: Vec<ModelDescriptor>,
    pub cameras: Vec<CameraDescriptor>,
    pub lights: Vec<LightDescriptor>,
}

/// Counts the import tasks that are still running on the rayon pool.
///
/// `rayon::ThreadPool` does **not** join its worker threads when it is dropped,
/// so the `Receiver` in [`Importer`] would otherwise be torn down while an
/// import was still running and that task's result would be silently lost.
/// [`Importer`]'s `Drop` waits on this counter so the channel outlives every
/// task it spawned.
#[derive(Default)]
struct InFlightTasks {
    count: Mutex<usize>,
    idle: Condvar,
}

impl InFlightTasks {
    /// Locks the counter, tolerating poisoning.
    ///
    /// A poisoned lock only means some task panicked while holding it; the
    /// count is still meaningful, and `Drop` must never panic.
    fn lock(&self) -> std::sync::MutexGuard<'_, usize> {
        self.count.lock().unwrap_or_else(|err| err.into_inner())
    }

    fn started(&self) {
        *self.lock() += 1;
    }

    fn finished(&self) {
        let mut count = self.lock();
        *count = count.saturating_sub(1);
        if *count == 0 {
            self.idle.notify_all();
        }
    }

    fn wait_until_idle(&self) {
        let mut count = self.lock();
        while *count > 0 {
            count = self.idle.wait(count).unwrap_or_else(|err| err.into_inner());
        }
    }
}

/// Decrements the in-flight count when dropped, including while unwinding from
/// a panic, so a panicking import task cannot stall [`Importer`]'s `Drop`.
struct InFlightGuard(Arc<InFlightTasks>);

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.0.finished();
    }
}

pub struct Importer {
    /// Declared first so it is dropped first: Rust drops fields in declaration
    /// order, and tearing down the pool before the channel gives every spawned
    /// task the best chance to deliver its result. See [`InFlightTasks`] for
    /// why field order alone is not enough.
    pool: rayon::ThreadPool,
    queued_tasks: VecDeque<ImportTask>,
    result_sender: mpsc::Sender<ImportResult>,
    result_receiver: Mutex<mpsc::Receiver<ImportResult>>,
    in_flight: Arc<InFlightTasks>,
}

impl Drop for Importer {
    /// Blocks until every spawned import task has finished.
    ///
    /// This runs before any field is dropped, so the result channel is still
    /// alive and the tasks can hand over their results instead of failing to
    /// send. The wait is bounded by how long an import takes; dropping an
    /// `Importer` during a large import will block the caller until it
    /// completes.
    fn drop(&mut self) {
        self.in_flight.wait_until_idle();
    }
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
            pool,
            queued_tasks: VecDeque::new(),
            result_sender: sender,
            result_receiver: Mutex::new(receiver),
            in_flight: Arc::new(InFlightTasks::default()),
        }
    }

    pub fn register_task(&mut self, task: ImportTask) {
        self.queued_tasks.push_back(task);
    }

    pub fn update(&mut self) -> Vec<ImportResult> {
        let mut results = Vec::new();

        while let Ok(result) = self.result_receiver.lock().unwrap().try_recv() {
            results.push(result);
        }

        while let Some(task_desc) = self.queued_tasks.pop_front() {
            let sender = self.result_sender.clone();
            let in_flight = Arc::clone(&self.in_flight);

            // Count the task before spawning it. `ThreadPool::spawn` only queues
            // the closure, so a task may not have started running yet; counting
            // inside the closure would let `Drop` see an empty queue and race
            // ahead of pending work.
            self.in_flight.started();

            self.pool.spawn(move || {
                let _guard = InFlightGuard(in_flight);

                let result = match task_desc {
                    ImportTask::Gltf { file_path, task } => {
                        let gltf_result = GltfImporter::import(GltfImportTask {
                            file: file_path,
                            import: task,
                        });

                        ImportResult {
                            models: gltf_result.models,
                            cameras: gltf_result.cameras,
                            lights: gltf_result.lights,
                        }
                    }
                };

                // Only reachable if the `Importer` is being torn down without
                // waiting, which `Drop` prevents; keep it visible rather than
                // discarding a result we cannot explain.
                if sender.send(result).is_err() {
                    log::warn!("Import result discarded: result channel already closed");
                }
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn wait_until_idle_returns_immediately_when_nothing_started() {
        let in_flight = InFlightTasks::default();
        in_flight.wait_until_idle();
    }

    #[test]
    fn wait_until_idle_blocks_until_every_task_finished() {
        let in_flight = Arc::new(InFlightTasks::default());
        let started = Instant::now();

        for _ in 0..2 {
            in_flight.started();
            let in_flight = Arc::clone(&in_flight);
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(50));
                let _guard = InFlightGuard(in_flight);
            });
        }

        in_flight.wait_until_idle();

        // It waited for the two 50ms tasks instead of returning straight away.
        assert!(
            started.elapsed() >= Duration::from_millis(50),
            "wait_until_idle returned before the tasks finished"
        );
        assert_eq!(*in_flight.lock(), 0);
    }

    #[test]
    fn guard_releases_the_count_when_a_task_panics() {
        let in_flight = Arc::new(InFlightTasks::default());
        let (tx, rx) = mpsc::channel();

        let panicking = Arc::clone(&in_flight);
        std::thread::spawn(move || {
            panicking.started();
            let _guard = InFlightGuard(panicking);
            let _ = tx.send(());
            panic!("task blew up after being counted as in flight");
        });

        rx.recv().expect("task should have started");
        // Would hang forever without the guard's unwinding cleanup.
        in_flight.wait_until_idle();
    }

    /// Regression test for the dropped-result bug this change fixes.
    ///
    /// Queueing far more work than the single worker can run immediately means
    /// most tasks are still pending when `drop` is called. `rayon::ThreadPool`
    /// does not join its threads, so without the in-flight wait these tasks
    /// would finish after the result channel was gone and their results would
    /// be silently lost.
    #[test]
    fn dropping_an_importer_waits_for_queued_tasks() {
        const TASKS: usize = 200;

        let mut importer = Importer::new(1);
        let in_flight = Arc::clone(&importer.in_flight);

        for i in 0..TASKS {
            importer.register_task(ImportTask::Gltf {
                // Unresolvable, so the import itself fails fast and the test
                // stays independent of glTF assets; the task is still spawned,
                // counted and accounted for like any other.
                file_path: format!("missing-{i}.gltf"),
                task: GltfImport::WholeFile,
            });
        }
        importer.update();

        drop(importer);

        assert_eq!(
            *in_flight.lock(),
            0,
            "drop returned while {TASKS} tasks were still running"
        );
    }
}
