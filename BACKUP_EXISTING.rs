
pub trait ImportTask<ImportData> {
    fn new(file: String, import: ImportData) -> Self
    where
        Self: Sized;

    fn begin(&mut self)
    where
        Self: Sized;

    fn is_done(&self)
    where
        Self: Sized;

    fn finish(&mut self) -> Vec<Event>
    where
        Self: Sized;
}

struct Test;
impl ImportTask<GltfImport> for Test {
    fn new(file: String, import: GltfImport) -> Self {
        todo!()
    }

    fn begin(&mut self) {
        todo!()
    }

    fn is_done(&self) {
        todo!()
    }

    fn finish(&mut self) -> Vec<Event> {
        todo!()
    }
}

struct Test2;
impl ImportTask<String> for Test2 {
    fn new(file: String, import: String) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn begin(&mut self)
    where
        Self: Sized,
    {
        todo!()
    }

    fn is_done(&self)
    where
        Self: Sized,
    {
        todo!()
    }

    fn finish(&mut self) -> Vec<Event>
    where
        Self: Sized,
    {
        todo!()
    }
}

pub type Y = Box<dyn Future<Output = (Vec<ModelDescriptor>, Vec<CameraDescriptor>)> + Send>;

trait X {
    fn into_future(
        self,
    ) -> Y;
}

impl X for GltfImportTask {
    fn into_future(
        self,
    ) -> Y {
        Box::new(async {
            let result = GltfImporter::import(self).await;

            (result.models, result.cameras)
        })
    }
}

async fn x() {
    let tasks: Vec<Box<dyn X>> = vec![
        Box::new(GltfImportTask {
            file: "Test0".to_string(),
            import: GltfImport::WholeFile,
        }),
        Box::new(GltfImportTask {
            file: "Test1".to_string(),
            import: GltfImport::WholeFile,
        }),
        Box::new(GltfImportTask {
            file: "Test2".to_string(),
            import: GltfImport::WholeFile,
        }),
    ];

    let simultaneous_tasks = 1;
    let mut running_tasks: Vec<Y> = Vec::new();

    let mut models = Vec::new();
    let mut cameras = Vec::new();

    while !tasks.is_empty() {
        for task in running_tasks {
            let x = task.as_ref();
        }

        if running_tasks.len() >= simultaneous_tasks {
            continue;
        }

        let next_task = tasks.remove(0);
        let next_future = next_task.into_future();

        running_tasks.push(next_future);
    }

    todo!()
}

// TODO: Rename module

pub struct Loader {
    simultaneous_tasks: u16,
    task_queue: Vec<ImportTask>,
}

impl Loader {
    pub fn new(simultaneous_tasks: u16) -> Self {
        Self {
            simultaneous_tasks: simultaneous_tasks,
            task_queue: Vec::new(),
        }
    }

    pub fn enqueue_task(&mut self, task: ImportTask) {
        self.task_queue.push(task);
    }

    pub fn update(&mut self) {
        let task = self.task_queue.remove(0);

        match task {
            ImportTask::GLTF { file, import } => {
                let x = GltfImporter::import(GltfImportTask { file, import });
            }
        }
    }
}
