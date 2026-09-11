use orbital::app::Module;
use orbital::ecs::{System, World};
use orbital::ecs_bridge::ImportQueueResource;
use orbital::importer::{ImportTask, gltf::GltfImport};

pub struct ModelModule;

impl Module for ModelModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Queue a glTF import — Cubes.glb has multiple colored cubes
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Models/Cubes.glb".into(),
                task: GltfImport::WholeFile,
            });
        }

        vec![] // No per-frame systems — importer handles entity spawning
    }
}
