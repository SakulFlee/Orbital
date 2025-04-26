use std::{
    error::Error,
    sync::Arc,
    thread::{self},
};

use camera::CameraDescriptor;
use cgmath::{EuclideanSpace, Point3, Vector3};
use crossbeam_channel::TryRecvError;
use easy_gltf::{Projection, Scene};
use loader::{Loader, LoaderError};
use log::{debug, error, warn};
use model::{MaterialShaderDescriptor, MeshDescriptor, ModelDescriptor, Transform};
use pbr_material_shader::PBRMaterialDescriptor;
use texture::{TextureDescriptor, TextureSize};
use vertex::Vertex;
use wgpu::{Color, TextureFormat, TextureUsages};

mod identifier;
pub use identifier::*;

mod worker;
pub use worker::*;

mod worker_mode;
pub use worker_mode::*;
use world_change::WorldChange;

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
            let mut model_descriptor: ModelDescriptor = Self::from_gltf_model(gltf_model);

            // If a model transform is given, apply it to all model descriptors.
            if let Some(model_transform) = option_model_transforms {
                model_descriptor.transforms = vec![*model_transform];
            }

            let world_change = WorldChange::SpawnModel(model_descriptor);

            world_changes.push(world_change);
        }

        // TODO: Lights
        // for gltf_light in &gltf_scene.lights {
        //     let light_descriptor: LightDescriptor = gltf_light.into();
        //     let world_change = WorldChange::SpawnLight(light_descriptor);

        //     world_changes.push(world_change);
        // }

        for gltf_camera in &gltf_scene.cameras {
            let camera_descriptor = Self::camera_from_gltf(gltf_camera);
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

    fn mesh_from_gltf(gltf_model: &easy_gltf::Model) -> MeshDescriptor {
        let vertices = gltf_model
            .vertices()
            .iter()
            .map(|vertex| Into::<Vertex>::into(*vertex))
            .collect::<Vec<Vertex>>();

        let indices = gltf_model
            .indices()
            .expect("Trying to load glTF model without Indices!")
            .clone();

        MeshDescriptor::new(vertices, indices)
    }

    fn material_from_gltf(value: &easy_gltf::Material) -> PBRMaterialDescriptor {
        fn rgb_to_rgba(data: &[u8]) -> Vec<u8> {
            data.chunks(3)
                .map(|x| [x[0], x[1], x[2], 255])
                .collect::<Vec<_>>()
                .concat()
        }

        // TODO: Unused
        fn convert_factor_to_u8(factor: f32) -> u8 {
            const U8_MIN_AS_F32: f32 = u8::MIN as f32;
            const U8_MAX_AS_F32: f32 = u8::MAX as f32;

            let unclamped = factor * U8_MAX_AS_F32;
            let clamped = unclamped.clamp(U8_MIN_AS_F32, U8_MAX_AS_F32);

            clamped as u8
        }

        let normal = value
            .normal
            .as_ref()
            .map(|x| {
                let pixels = rgb_to_rgba(&x.texture);
                let (width, height) = x.texture.dimensions();
                TextureDescriptor::Data {
                    pixels,
                    size: TextureSize {
                        width,
                        height,
                        ..Default::default()
                    },
                    usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                    format: TextureFormat::Rgba8UnormSrgb,
                }
            })
            .unwrap_or(TextureDescriptor::uniform_rgba_black());

        let albedo = value
            .pbr
            .base_color_texture
            .as_ref()
            .map(|x| {
                let (width, height) = x.dimensions();
                TextureDescriptor::Data {
                    pixels: x.to_vec(),
                    size: TextureSize {
                        width,
                        height,
                        ..Default::default()
                    },
                    usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                    format: TextureFormat::Rgba8UnormSrgb,
                }
            })
            .unwrap_or_else(|| {
                TextureDescriptor::uniform_rgba_color(Color {
                    r: value.pbr.base_color_factor.x as f64,
                    g: value.pbr.base_color_factor.y as f64,
                    b: value.pbr.base_color_factor.z as f64,
                    a: value.pbr.base_color_factor.w as f64,
                })
            });

        let metallic = value
            .pbr
            .metallic_texture
            .as_ref()
            .map(|x| TextureDescriptor::Data {
                pixels: x.as_raw().to_vec(),
                size: TextureSize {
                    width: x.width(),
                    height: x.height(),
                    ..Default::default()
                },
                usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                format: TextureFormat::R8Unorm,
            })
            .unwrap_or_else(|| {
                TextureDescriptor::uniform_luma_value(value.pbr.metallic_factor as f64)
            });

        let roughness = value
            .pbr
            .roughness_texture
            .as_ref()
            .map(|x| TextureDescriptor::Data {
                pixels: x.as_raw().to_vec(),
                size: TextureSize {
                    width: x.width(),
                    height: x.height(),
                    ..Default::default()
                },
                usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                format: TextureFormat::R8Unorm,
            })
            .unwrap_or(TextureDescriptor::uniform_luma_value(
                value.pbr.roughness_factor as f64,
            ));

        let occlusion = value
            .occlusion
            .as_ref()
            .map(|x| TextureDescriptor::Data {
                pixels: x.texture.as_raw().to_vec(),
                size: TextureSize {
                    width: x.texture.width(),
                    height: x.texture.height(),
                    ..Default::default()
                },
                usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                format: TextureFormat::R8Unorm,
            })
            .unwrap_or(TextureDescriptor::uniform_luma_white());

        let emissive = value
            .emissive
            .texture
            .as_ref()
            .map(|x| TextureDescriptor::Data {
                pixels: rgb_to_rgba(x.as_raw()),
                size: TextureSize {
                    width: x.width(),
                    height: x.height(),
                    ..Default::default()
                },
                usages: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                format: TextureFormat::Rgba8UnormSrgb,
            })
            .unwrap_or(TextureDescriptor::uniform_rgba_black());

        PBRMaterialDescriptor {
            name: value.name.clone(),
            normal: normal,
            albedo: albedo,
            albedo_factor: Vector3::new(
                value.pbr.base_color_factor.x,
                value.pbr.base_color_factor.y,
                value.pbr.base_color_factor.z,
            ),
            metallic: metallic,
            metallic_factor: value.pbr.metallic_factor,
            roughness: roughness,
            roughness_factor: value.pbr.roughness_factor,
            occlusion: occlusion,
            emissive: emissive,
            custom_material_shader: None,
        }
    }

    fn camera_from_gltf(value: &easy_gltf::Camera) -> CameraDescriptor {
        let identifier = if let Some(name) = &value.name {
            name.clone()
        } else {
            String::from("Unnamed glTF Camera")
        };

        let forward = value.forward();
        let yaw = forward.y.atan2(forward.x);
        let pitch = -forward.z.asin();

        match value.projection {
            Projection::Perspective {
                yfov: fovy,
                aspect_ratio,
            } => CameraDescriptor {
                label: identifier,
                position: Point3::from_vec(value.position()),
                yaw,
                pitch,
                fovy: fovy.0,
                aspect: aspect_ratio.unwrap_or(16.0 / 9.0),
                near: value.znear,
                far: value.zfar,
                ..Default::default()
            },
            Projection::Orthographic { scale: _ } => unimplemented!(),
        }
    }

    fn from_gltf_model(gltf_model: &easy_gltf::Model) -> ModelDescriptor {
        let label = gltf_model
            .mesh_name()
            .map(|x| x.to_string())
            .unwrap_or(String::from("unlabelled glTF Model"));

        let mesh_descriptor = Self::mesh_from_gltf(gltf_model);
        let pbr_material_descriptor = Self::material_from_gltf(&gltf_model.material());
        let material_descriptor: MaterialShaderDescriptor = pbr_material_descriptor.into();

        ModelDescriptor {
            label,
            mesh: Arc::new(mesh_descriptor),
            materials: vec![Arc::new(material_descriptor)],
            transforms: vec![Transform::default()], // TODO: This only works because vertices seem to already be offset by the correct amount for their local space. We should find out if this is from easy_gltf, or, encoded into glTF directly by Blender. Either way, it might be best to "re-local-ize" the vertices to reduce number overhead and properly use transforms.
            #[cfg(debug_assertions)]
            render_bounding_box: false,
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
                        .send(Err(e))
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
                    scene_light_map: _,
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
                            .map(Self::from_gltf_model)
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

                    // // Lights
                    // if let Some(scene_lights) = scene_light_map {
                    //     let light_world_changes = scene_lights
                    //         .into_iter()
                    //         // Find each scene specified to be selected by the key (k), pass on the object selector vec (v)
                    //         .map(|(k, v)| {
                    //             let scene = match k {
                    //                 GLTFIdentifier::Id(id) => gltf_scenes.get(id),
                    //                 GLTFIdentifier::Label(label) => gltf_scenes
                    //                     .iter()
                    //                     .find(|x| x.name.as_ref().is_some_and(|x| x == label)),
                    //             };

                    //             (scene, v)
                    //         })
                    //         // Remove any scenes that could not be found
                    //         .filter_map(|x| {
                    //             if let Some(scene) = x.0 {
                    //                 Some((scene, x.1))
                    //             } else {
                    //                 None
                    //             }
                    //         })
                    //         // Extract any objects mentioned (v) from the scene (k)
                    //         .flat_map(|(k, v)| {
                    //             v.iter()
                    //                 .map(|x| match x {
                    //                     GLTFIdentifier::Id(id) => k.lights.get(id),
                    //                     GLTFIdentifier::Label(label) => k.lights.iter().find(|y| {
                    //                         let name = match y {
                    //                             Light::Directional {
                    //                                 name,
                    //                                 direction: _,
                    //                                 color: _,
                    //                                 intensity: _,
                    //                             } => name,
                    //                             Light::Point {
                    //                                 name,
                    //                                 position: _,
                    //                                 color: _,
                    //                                 intensity: _,
                    //                             } => name,
                    //                             Light::Spot {
                    //                                 name,
                    //                                 position: _,
                    //                                 direction: _,
                    //                                 color: _,
                    //                                 intensity: _,
                    //                                 inner_cone_angle: _,
                    //                                 outer_cone_angle: _,
                    //                             } => name,
                    //                         };

                    //                         name.as_ref().is_some_and(|z| z == label)
                    //                     }),
                    //                 })
                    //                 .collect::<Vec<_>>()
                    //         })
                    //         // Remove any objects that could not be found
                    //         .flatten()
                    //         // Convert to descriptors
                    //         .map(LightDescriptor::from)
                    //         // Convert to world changes
                    //         .map(WorldChange::SpawnLight)
                    //         .collect::<Vec<_>>();

                    // world_changes.extend(light_world_changes);
                    // }

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
                            .map(Self::camera_from_gltf)
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

    fn finish_processing(&mut self) -> Result<Vec<WorldChange>, Box<dyn Error + Send + Sync>> {
        if !self.is_done_processing() {
            return Err(Box::new(LoaderError::NotDoneProcessing));
        }

        let worker = self.worker.take().unwrap();

        // Wait for the thread to be done, just in case.
        worker
            .worker
            .join()
            .expect("Worker thread didn't exit correctly!");

        let result: Result<Vec<WorldChange>, Box<dyn Error + Send + Sync>> =
            match worker.receiver.try_recv() {
                Ok(x) => x,
                Err(e) => match e {
                    TryRecvError::Empty => return Err(Box::new(LoaderError::EmptyResult)),
                    TryRecvError::Disconnected => {
                        return Err(Box::new(LoaderError::DisconnectedChannelNoData));
                    }
                },
            };

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
