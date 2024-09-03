use std::{
    ops::{Range, RangeBounds},
    slice::SliceIndex,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use cgmath::{EuclideanSpace, Point3};
use crossbeam_channel::Receiver;
use easy_gltf::{Camera, Light, Material, Model, Projection, Scene};
use hashbrown::HashMap;
use log::warn;

use crate::{
    error::Error,
    game::WorldChange,
    resources::{
        descriptors::{
            CameraDescriptor, InstanceDescriptor, Instancing, LightDescriptor, MaterialDescriptor,
            MeshDescriptor, ModelDescriptor, TextureDescriptor,
        },
        realizations::Vertex,
    },
    util::rgb_to_rgba,
};

use super::Loader;

#[derive(Debug, Clone)]
pub enum GLTFObjectType {
    Model,
    Light,
    Camera,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GLTFIdentifier {
    /// The numerical id, starting at zero, of the location of the object
    /// inside the glTF file.  
    /// Indices cannot be skipped and the order will be preserved.
    /// Thus, the first object will always be the 0th index.
    Id(usize),
    /// Label/Name of the object inside the glTF file.
    ///
    /// ⚠️ Labels in glTF files are an optional feature and must be  
    ///     supported by the glTF file / exporter.
    ///
    /// Less performant than [GLTFIdentifier::Id] as it needs to
    /// search through all entries until the label is found or no
    /// more objects are to be inspected.  
    /// However, if used in a [Loader], performance can be ignored.
    Label(&'static str),
}

impl GLTFIdentifier {
    pub fn ranged_id(start: usize, end: usize) -> Vec<Self> {
        if start > end {
            panic!("Ranged start cannot be bigger than end!");
        }

        let mut v = Vec::new();

        for i in start..=end {
            v.push(GLTFIdentifier::Id(i));
        }

        v
    }
}

#[derive(Debug, Clone)]
pub enum GLTFLoaderMode {
    /// Loads everything contained in the GLTF File.
    /// This includes all scenes, though scenes won't be imported themselves.
    /// Meaning, everything inside each scene will be loaded into the same
    /// instance.
    LoadEverything,
    /// Loads one or multiple specific scenes with everything in it.
    /// This includes any and all resources inside any of the scenes.
    /// Scenes aren't directly supported by the engine, thus everything
    /// will be loaded into the same instance.
    ///
    /// To trigger a "scene change" like in other engines, see:  
    /// [WorldChange::CleanWorld].
    ///
    /// [WorldChange::CleanWorld]: crate::game::WorldChange::CleanWorld
    LoadScenes {
        scene_identifiers: Vec<GLTFIdentifier>,
    },
    /// Loads a specific subset of resources (models, lights, cameras).  
    /// The first [GLTFIdentifier] is for identifying the scene.  
    /// The second [GLTFIdentifier] is for identifying the actual resource.
    ///
    /// You can import only a given resource (e.g. models) by only filling out
    /// the corresponding map.
    LoadSpecific {
        scene_model_map: Option<HashMap<GLTFIdentifier, Vec<GLTFIdentifier>>>,
        scene_light_map: Option<HashMap<GLTFIdentifier, Vec<GLTFIdentifier>>>,
        scene_camera_map: Option<HashMap<GLTFIdentifier, Vec<GLTFIdentifier>>>,
    },
}

#[derive(Debug)]
pub struct GLTFWorker {
    receiver: Receiver<Result<Vec<WorldChange>, Error>>,
    worker: JoinHandle<()>,
}

#[derive(Debug)]
pub struct GLTFLoader {
    file_path: &'static str,
    mode: GLTFLoaderMode,
    worker: Option<GLTFWorker>,
}

impl GLTFLoader {
    pub fn new(file_path: &'static str, mode: GLTFLoaderMode) -> Self
    where
        Self: Sized,
    {
        Self {
            file_path: file_path.into(),
            mode,
            worker: None,
        }
    }

    fn loader_load_objects_from_scene(
        gltf_scene: &Scene,
        identifiers: Vec<GLTFIdentifier>,
        object_type: GLTFObjectType,
    ) -> Vec<WorldChange> {
        let (ids, labels): (Vec<_>, Vec<_>) = identifiers.into_iter().partition(|x| match x {
            GLTFIdentifier::Id(_) => true,
            GLTFIdentifier::Label(_) => false,
        });

        let actual_ids: Vec<usize> = ids
            .into_iter()
            .map(|x| {
                if let GLTFIdentifier::Id(id) = x {
                    id
                } else {
                    unreachable!()
                }
            })
            .collect();

        let actual_labels: Vec<&str> = labels
            .into_iter()
            .map(|x| {
                if let GLTFIdentifier::Label(label) = x {
                    label
                } else {
                    unreachable!()
                }
            })
            .collect();

        let world_changes: Vec<_> = match object_type {
            GLTFObjectType::Model => gltf_scene
                .models
                .iter()
                .enumerate()
                .filter(|(id, model)| {
                    actual_ids.contains(id)
                        || model
                            .mesh_name()
                            .is_some_and(|label| actual_labels.contains(&label))
                })
                .map(|(_, model)| {
                    let model_descriptor: ModelDescriptor = model.into();
                    let world_change = WorldChange::SpawnModel(model_descriptor);

                    world_change
                })
                .collect(),
            GLTFObjectType::Light => gltf_scene
                .lights
                .iter()
                .enumerate()
                .filter(|(id, light)| {
                    if actual_ids.contains(id) {
                        true
                    } else {
                        match light {
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
                        }
                        .as_ref()
                        .is_some_and(|label| actual_labels.contains(&label.as_str()))
                    }
                })
                .map(|(_, light)| {
                    let light_descriptor: LightDescriptor = light.into();
                    let world_change = WorldChange::SpawnLight(light_descriptor);

                    world_change
                })
                .collect(),
            GLTFObjectType::Camera => gltf_scene
                .cameras
                .iter()
                .enumerate()
                .filter(|(id, camera)| {
                    actual_ids.contains(id)
                        || camera
                            .name
                            .as_ref()
                            .is_some_and(|label| actual_labels.contains(&label.as_str()))
                })
                .map(|(_, camera)| {
                    let camera_descriptor: CameraDescriptor = camera.into();
                    let world_change = WorldChange::SpawnCamera(camera_descriptor);

                    world_change
                })
                .collect(),
        };
        world_changes
    }

    fn loader_gltf_scene_to_world_changes(gltf_scene: &Scene) -> Vec<WorldChange> {
        let mut world_changes = Vec::new();

        // TODO: Parallelise!

        for gltf_model in &gltf_scene.models {
            let model_descriptor: ModelDescriptor = gltf_model.into();
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

    fn loader_load_everything(gltf_scenes: &[Scene]) -> Vec<WorldChange> {
        let mut world_changes = Vec::new();

        // TODO: Parallelise!

        for gltf_scene in gltf_scenes {
            let scene_world_changes = Self::loader_gltf_scene_to_world_changes(gltf_scene);

            world_changes.extend(scene_world_changes);
        }

        world_changes
    }

    fn loader_load_scene(gltf_scenes: &[Scene], identifier: &GLTFIdentifier) -> Vec<WorldChange> {
        // TODO: Parallelise!

        match identifier {
            GLTFIdentifier::Id(id) => gltf_scenes
                .into_iter()
                .enumerate()
                .filter_map(|(i, x)| i.eq(id).then_some(x))
                .map(|scene| Self::loader_gltf_scene_to_world_changes(scene))
                .flatten()
                .collect(),
            GLTFIdentifier::Label(labels) => gltf_scenes
                .into_iter()
                .filter(|x| x.name.is_some())
                .filter(|x| labels.contains(&x.name.as_ref().unwrap().as_str()))
                .map(|x| Self::loader_gltf_scene_to_world_changes(x))
                .flatten()
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

        let (sender, receiver) = crossbeam_channel::unbounded();

        let file_path = self.file_path;
        let mode = self.mode.clone();

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
                GLTFLoaderMode::LoadEverything => Self::loader_load_everything(&gltf_scenes),
                GLTFLoaderMode::LoadScenes {
                    scene_identifiers: identifiers,
                } => {
                    let mut world_changes = Vec::new();

                    for identifier in identifiers {
                        let scene_world_change = Self::loader_load_scene(&gltf_scenes, &identifier);

                        world_changes.extend(scene_world_change);
                    }

                    world_changes
                }
                GLTFLoaderMode::LoadSpecific {
                    scene_model_map,
                    scene_light_map,
                    scene_camera_map,
                } => {
                    let mut world_changes = Vec::new();

                    // Models
                    if let Some(scene_models) = scene_model_map {
                        let model_world_changes = scene_models
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
                            .map(|(k, v)| {
                                v.iter()
                                    .map(|x| match x {
                                        GLTFIdentifier::Id(id) => k.models.get(*id),
                                        GLTFIdentifier::Label(label) => k.models.iter().find(|y| {
                                            y.mesh_name().as_ref().is_some_and(|z| z == label)
                                        }),
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .flatten()
                            // Remove any objects that could not be found
                            .filter_map(|x| x)
                            // Convert to descriptors
                            .map(|x| ModelDescriptor::from(x))
                            // Convert to world changes
                            .map(|x| WorldChange::SpawnModel(x))
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
                            .map(|(k, v)| {
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
                            .flatten()
                            // Remove any objects that could not be found
                            .filter_map(|x| x)
                            // Convert to descriptors
                            .map(|x| LightDescriptor::from(x))
                            // Convert to world changes
                            .map(|x| WorldChange::SpawnLight(x))
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
                            .map(|(k, v)| {
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
                            .flatten()
                            // Remove any objects that could not be found
                            .filter_map(|x| x)
                            // Convert to descriptors
                            .map(|x| CameraDescriptor::from(x))
                            // Convert to world changes
                            .map(|x| WorldChange::SpawnCamera(x))
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

        worker
            .receiver
            .recv()
            .map_err(|e| Error::CrossbeamRecvError(e))?
    }
}

impl From<&Model> for ModelDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let material_descriptor = gltf_model.material().as_ref().into();
        let mesh_descriptor = gltf_model.into();

        ModelDescriptor::FromDescriptors(
            mesh_descriptor,
            material_descriptor,
            Instancing::Single(InstanceDescriptor::default()),
        )
    }
}

impl From<&Model> for MeshDescriptor {
    fn from(gltf_model: &Model) -> Self {
        let vertices = gltf_model
            .vertices()
            .iter()
            .map(|vertex| Into::<Vertex>::into(*vertex))
            .collect::<Vec<Vertex>>();

        let indices = gltf_model
            .indices()
            .expect("Trying to load glTF model without Indices!")
            .clone();

        MeshDescriptor { vertices, indices }
    }
}

impl From<&Material> for MaterialDescriptor {
    fn from(value: &Material) -> Self {
        let normal = value
            .normal
            .as_ref()
            .map(|x| {
                let data = rgb_to_rgba(&x.texture);
                TextureDescriptor::StandardSRGBAu8Data(data, x.texture.dimensions().into())
            })
            .unwrap_or(TextureDescriptor::UNIFORM_BLACK);

        let albedo = value
            .pbr
            .base_color_texture
            .as_ref()
            .map(|x| {
                TextureDescriptor::StandardSRGBAu8Data(x.as_raw().to_vec(), x.dimensions().into())
            })
            .unwrap_or(TextureDescriptor::UniformColor(
                value.pbr.base_color_factor.map(|x| (x * 255.0) as u8),
            ));

        let metallic = value
            .pbr
            .metallic_texture
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.as_raw().to_vec(),
                size: x.dimensions().into(),
            })
            .unwrap_or(TextureDescriptor::UniformLuma {
                data: (value.pbr.metallic_factor * 255.0) as u8,
            });

        let roughness = value
            .pbr
            .roughness_texture
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.as_raw().to_vec(),
                size: x.dimensions().into(),
            })
            .unwrap_or(TextureDescriptor::UniformLuma {
                data: (value.pbr.roughness_factor * 255.0) as u8,
            });

        let occlusion = value
            .occlusion
            .as_ref()
            .map(|x| TextureDescriptor::Luma {
                data: x.texture.to_vec(),
                size: x.texture.dimensions().into(),
            })
            .unwrap_or(TextureDescriptor::UNIFORM_LUMA_WHITE);

        let emissive = value
            .emissive
            .texture
            .as_ref()
            .map(|x| {
                TextureDescriptor::StandardSRGBAu8Data(
                    rgb_to_rgba(x.as_raw()),
                    x.dimensions().into(),
                )
            })
            .unwrap_or(TextureDescriptor::UNIFORM_WHITE);

        // TODO: Include factors!
        // Factors != Values

        Self::PBR {
            normal,
            albedo,
            metallic,
            roughness,
            occlusion,
            emissive,
        }
    }
}

impl From<&Light> for LightDescriptor {
    fn from(value: &Light) -> Self {
        match value {
            Light::Point {
                name,
                position,
                color,
                intensity,
            } => LightDescriptor::PointLight {
                position: position.clone(),
                color: color.clone(),
            },
            Light::Directional {
                name,
                direction,
                color,
                intensity,
            } => unimplemented!(),
            Light::Spot {
                name,
                position,
                direction,
                color,
                intensity,
                inner_cone_angle,
                outer_cone_angle,
            } => unimplemented!(),
        }
    }
}

impl From<&Camera> for CameraDescriptor {
    fn from(value: &Camera) -> Self {
        let identifier = if let Some(name) = &value.name {
            name.clone()
        } else {
            String::from("Unnamed glTF Camera")
        };

        // TODO: Validate this calculation!
        let forward = value.forward();
        let yaw = forward.y.atan2(forward.x);
        let pitch = -forward.z.asin();

        match value.projection {
            Projection::Perspective {
                yfov: fovy,
                aspect_ratio,
            } => CameraDescriptor {
                identifier,
                position: Point3::from_vec(value.position()),
                yaw,
                pitch,
                fovy: fovy.0,
                aspect: aspect_ratio.unwrap_or(16.0 / 9.0),
                near: value.znear,
                far: value.zfar,
                ..Default::default()
            },
            Projection::Orthographic { scale } => unimplemented!(),
        }
    }
}

#[cfg(test)]
const TEST_FILE_PATH: &'static str = "../Assets/Models/PBR_Spheres.glb";
#[cfg(test)]
const TEST_FILE_WORLD_CHANGES: usize = 121;

#[test]
fn load_everything() {
    let mut loader = GLTFLoader::new(TEST_FILE_PATH, GLTFLoaderMode::LoadEverything);

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();
    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), TEST_FILE_WORLD_CHANGES);
}

#[test]
fn load_scene_id() {
    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFLoaderMode::LoadScenes {
            scene_identifiers: vec![
                GLTFIdentifier::Id(0),
                GLTFIdentifier::Id(1),
                GLTFIdentifier::Id(2),
            ],
        },
    );

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), TEST_FILE_WORLD_CHANGES);
}

#[test]
fn load_scene_label() {
    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFLoaderMode::LoadScenes {
            scene_identifiers: vec![GLTFIdentifier::Label("Scene")],
        },
    );

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), TEST_FILE_WORLD_CHANGES);
}

#[test]
fn load_scene_specific() {
    let mut models = HashMap::new();
    models.insert(
        GLTFIdentifier::Label("Scene"),
        GLTFIdentifier::ranged_id(0, 2),
    );

    let mut loader = GLTFLoader::new(
        TEST_FILE_PATH,
        GLTFLoaderMode::LoadSpecific {
            scene_model_map: Some(models),
            scene_light_map: None,
            scene_camera_map: None,
        },
    );

    println!("Begin processing glTF file: {}", TEST_FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    // Wait until done
    while !loader.is_done_processing() {}

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());

    if result.is_err() {
        panic!("Result is not expected: {:?}", result);
    }

    let world_changes = result.unwrap();
    println!(
        "Finished processing! {} WorldChange's generated!",
        world_changes.len()
    );
    assert_eq!(world_changes.len(), 3);
}
