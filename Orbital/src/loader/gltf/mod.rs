use std::thread::{self};

use easy_gltf::{Light, Scene};
use log::{debug, error, warn};

use crate::{
    error::Error,
    resources::descriptors::{CameraDescriptor, LightDescriptor, ModelDescriptor},
    transform::Transform,
    world::WorldChange,
};

use super::Loader;

mod identifier;
mod worker;
mod worker_mode;

pub use identifier::*;
pub use worker::*;
pub use worker_mode::*;

// Into's
mod into_camera_descriptor;
mod into_light_descriptor;
mod into_material_descriptor;
mod into_mesh_descriptor;
mod into_model_descriptor;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct GLTFLoader {
    file_path: &'static str,
    mode: GLTFWorkerMode,
    worker: Option<GLTFWorker>,
    model_transform: Option<Transform>,
}

impl GLTFLoader {
    pub fn new(
        file_path: &'static str,
        mode: GLTFWorkerMode,
        model_transform: Option<Transform>,
    ) -> Self
    where
        Self: Sized,
    {
        Self {
            file_path,
            mode,
            worker: None,
            model_transform,
        }
    }

    fn loader_gltf_scene_to_world_changes(
        gltf_scene: &Scene,
        option_model_transforms: &Option<Transform>,
    ) -> Vec<WorldChange> {
        let mut world_changes = Vec::new();

        // TODO: Parallelise!

        for gltf_model in &gltf_scene.models {
            let mut model_descriptor: ModelDescriptor = gltf_model.into();

            // If a model transform is given, apply it to all model descriptors.
            if let Some(model_transform) = option_model_transforms {
                model_descriptor.transforms = vec![*model_transform];
            }

            let world_change = WorldChange::SpawnModel(model_descriptor);

            world_changes.push(world_change);
        }

        for gltf_light in &gltf_scene.lights {
            let light_descriptor: LightDescriptor = gltf_light.into();
            let world_change = WorldChange::SpawnLight(light_descriptor);

            world_changes.push(world_change);
        }

        for gltf_camera in &gltf_scene.cameras {
            let camera_descriptor: CameraDescriptor = gltf_camera.into();
            let world_change = WorldChange::SpawnCamera(camera_descriptor);

            world_changes.push(world_change);
        }

        world_changes
    }

    fn loader_load_everything(
        gltf_scenes: &[Scene],
        option_model_transform: &Option<Transform>,
    ) -> Vec<WorldChange> {
        let mut world_changes = Vec::new();

        // TODO: Parallelise!

        for gltf_scene in gltf_scenes {
            let scene_world_changes =
                Self::loader_gltf_scene_to_world_changes(gltf_scene, option_model_transform);

            world_changes.extend(scene_world_changes);
        }

        world_changes
    }

    fn loader_load_scene(
        gltf_scenes: &[Scene],
        identifier: &GLTFIdentifier,
        option_model_transform: &Option<Transform>,
    ) -> Vec<WorldChange> {
        // TODO: Parallelise!

        match identifier {
            GLTFIdentifier::Id(id) => gltf_scenes
                .iter()
                .enumerate()
                .filter_map(|(i, x)| i.eq(id).then_some(x))
                .flat_map(|x| Self::loader_gltf_scene_to_world_changes(x, option_model_transform))
                .collect(),
            GLTFIdentifier::Label(labels) => gltf_scenes
                .iter()
                .filter(|x| x.name.is_some())
                .filter(|x| labels.contains(x.name.as_ref().unwrap().as_str()))
                .flat_map(|x| Self::loader_gltf_scene_to_world_changes(x, option_model_transform))
                .collect(),
        }
    }
}

impl Loader for GLTFLoader {
    fn begin_processing(&mut self) {
        if self.worker.is_some() {
            warn!("GLTFLoader is already processing!");
            return;
        }
        debug!("Started processing: {}@{:?}", self.file_path, self.mode);

        let (sender, receiver) = crossbeam_channel::unbounded();

        let file_path = self.file_path;
        let mode = self.mode.clone();
        let option_model_transform = self.model_transform.take();

        let worker = thread::spawn(move || {
            let gltf_scenes = match easy_gltf::load(file_path) {
                Ok(x) => x,
                Err(e) => {
                    sender
                        .send(Err(Error::GltfError(e)))
                        .expect("GLTFLoader failed sending error results to main process!");
                    return;
                }
            };

            let world_changes = match mode {
                GLTFWorkerMode::LoadEverything => {
                    Self::loader_load_everything(&gltf_scenes, &option_model_transform)
                }
                GLTFWorkerMode::LoadScenes {
                    scene_identifiers: identifiers,
                } => {
                    let mut world_changes = Vec::new();

                    for identifier in identifiers {
                        let scene_world_change = Self::loader_load_scene(
                            &gltf_scenes,
                            &identifier,
                            &option_model_transform,
                        );

                        world_changes.extend(scene_world_change);
                    }

                    world_changes
                }
                GLTFWorkerMode::LoadSpecific {
                    scene_model_map,
                    scene_light_map,
                    scene_camera_map,
                } => {
                    let mut world_changes = Vec::new();

                    // Models
                    if let Some(scene_models) = scene_model_map {
                        let mut models = scene_models
                            .into_iter()
                            // Find each scene specified to be selected by the key (k), pass on the object selector vec (v)
                            .map(|(k, v)| {
                                let scene = match k {
                                    GLTFIdentifier::Id(id) => gltf_scenes.get(id),
                                    GLTFIdentifier::Label(label) => gltf_scenes
                                        .iter()
                                        .find(|x| x.name.as_ref().is_some_and(|x| x == label)),
                                };

                                (scene, v)
                            })
                            // Remove any scenes that could not be found
                            .filter_map(|x| {
                                if let Some(scene) = x.0 {
                                    Some((scene, x.1))
                                } else {
                                    None
                                }
                            })
                            // Extract any objects mentioned (v) from the scene (k)
                            .flat_map(|(k, v)| {
                                v.iter()
                                    .map(|x| match x {
                                        GLTFIdentifier::Id(id) => k.models.get(*id),
                                        GLTFIdentifier::Label(label) => k.models.iter().find(|y| {
                                            y.mesh_name().as_ref().is_some_and(|z| z == label)
                                        }),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            // Remove any objects that could not be found
                            .flatten()
                            // Convert to descriptors
                            .map(ModelDescriptor::from)
                            .collect::<Vec<_>>();

                        if let Some(model_transform) = &option_model_transform {
                            models
                                .iter_mut()
                                .for_each(|x| x.transforms = vec![*model_transform]);
                        }

                        // Convert to world changes
                        let model_world_changes = models
                            .into_iter()
                            .map(WorldChange::SpawnModel)
                            .collect::<Vec<_>>();

                        world_changes.extend(model_world_changes);
                    }

                    // Lights
                    if let Some(scene_lights) = scene_light_map {
                        let light_world_changes = scene_lights
                            .into_iter()
                            // Find each scene specified to be selected by the key (k), pass on the object selector vec (v)
                            .map(|(k, v)| {
                                let scene = match k {
                                    GLTFIdentifier::Id(id) => gltf_scenes.get(id),
                                    GLTFIdentifier::Label(label) => gltf_scenes
                                        .iter()
                                        .find(|x| x.name.as_ref().is_some_and(|x| x == label)),
                                };

                                (scene, v)
                            })
                            // Remove any scenes that could not be found
                            .filter_map(|x| {
                                if let Some(scene) = x.0 {
                                    Some((scene, x.1))
                                } else {
                                    None
                                }
                            })
                            // Extract any objects mentioned (v) from the scene (k)
                            .flat_map(|(k, v)| {
                                v.iter()
                                    .map(|x| match x {
                                        GLTFIdentifier::Id(id) => k.lights.get(*id),
                                        GLTFIdentifier::Label(label) => k.lights.iter().find(|y| {
                                            let name = match y {
                                                Light::Directional {
                                                    name,
                                                    direction: _,
                                                    color: _,
                                                    intensity: _,
                                                } => name,
                                                Light::Point {
                                                    name,
                                                    position: _,
                                                    color: _,
                                                    intensity: _,
                                                } => name,
                                                Light::Spot {
                                                    name,
                                                    position: _,
                                                    direction: _,
                                                    color: _,
                                                    intensity: _,
                                                    inner_cone_angle: _,
                                                    outer_cone_angle: _,
                                                } => name,
                                            };

                                            name.as_ref().is_some_and(|z| z == label)
                                        }),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            // Remove any objects that could not be found
                            .flatten()
                            // Convert to descriptors
                            .map(LightDescriptor::from)
                            // Convert to world changes
                            .map(WorldChange::SpawnLight)
                            .collect::<Vec<_>>();

                        world_changes.extend(light_world_changes);
                    }

                    // Cameras
                    if let Some(scene_cameras) = scene_camera_map {
                        let camera_world_changes = scene_cameras
                            .into_iter()
                            // Find each scene specified to be selected by the key (k), pass on the object selector vec (v)
                            .map(|(k, v)| {
                                let scene = match k {
                                    GLTFIdentifier::Id(id) => gltf_scenes.get(id),
                                    GLTFIdentifier::Label(label) => gltf_scenes
                                        .iter()
                                        .find(|x| x.name.as_ref().is_some_and(|x| x == label)),
                                };

                                (scene, v)
                            })
                            // Remove any scenes that could not be found
                            .filter_map(|x| {
                                if let Some(scene) = x.0 {
                                    Some((scene, x.1))
                                } else {
                                    None
                                }
                            })
                            // Extract any objects mentioned (v) from the scene (k)
                            .flat_map(|(k, v)| {
                                v.iter()
                                    .map(|x| match x {
                                        GLTFIdentifier::Id(id) => k.cameras.get(*id),
                                        GLTFIdentifier::Label(label) => k
                                            .cameras
                                            .iter()
                                            .find(|y| y.name.as_ref().is_some_and(|z| z == label)),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            // Remove any objects that could not be found
                            .flatten()
                            // Convert to descriptors
                            .map(CameraDescriptor::from)
                            // Convert to world changes
                            .map(WorldChange::SpawnCamera)
                            .collect::<Vec<_>>();

                        world_changes.extend(camera_world_changes);
                    }

                    world_changes
                }
            };

            sender
                .send(Ok(world_changes))
                .expect("GLTFLoader failed sending successful results to main process!");
        });

        self.worker = Some(GLTFWorker { receiver, worker });
    }

    fn is_done_processing(&self) -> bool {
        if let Some(worker) = &self.worker {
            !worker.receiver.is_empty()
        } else {
            false
        }
    }

    fn finish_processing(&mut self) -> Result<Vec<WorldChange>, Error> {
        if !self.is_done_processing() {
            return Err(Error::NotDoneProcessing);
        }

        let worker = self.worker.take().unwrap();

        // Wait for the thread to be done, just in case.
        worker
            .worker
            .join()
            .expect("Worker thread didn't exit correctly!");

        let result = worker.receiver.recv().map_err(Error::CrossbeamRecvError)?;

        match &result {
            Ok(world_changes) => debug!(
                "Finished processing {}@{:?} successfully! Generated {} world changes.",
                self.file_path,
                self.mode,
                world_changes.len()
            ),
            Err(e) => error!(
                "Finished processing {}@{:?} with an error: {:?}",
                self.file_path, self.mode, e
            ),
        }

        result
    }
}
