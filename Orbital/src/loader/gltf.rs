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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GLTFIdentifier {
    /// The numerical id, starting at zero, of the location of the object.
    /// Indices cannot be skipped, meaning that an empty index == end of data.  
    ///
    /// Will be used to find objects very efficiently and fast by their index.
    ///
    /// ⚠️ Avoid using a [Range] like: `0..=usize::MAX` to load everything.  
    /// ⚠️ Instead, use [GLTFLoaderMode::LoadEverything] or [GLTFLoaderMode::LoadScenes]!
    Id(Vec<usize>),
    /// ⚠️ Labels (names) must be supported by the glTF file.
    /// ⚠️ It's an optional feature and is not always included!
    ///
    /// Will attempt searching for the label(s) and select them.
    /// Slower, compared to [GLTFIdentifier::Id].
    ///
    /// ⚠️ If a Label can't be found, it will be skipped. No error is produced.
    /// ⚠️ If a Label is listed multiple times, all will be selected.
    /// ⚠️ If nothing matches the Label(s), nothing is selected.
    Label(Vec<&'static str>),
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
    LoadScenes { identifiers: Vec<GLTFIdentifier> },
    /// Loads a specific subset of resources (models, lights, cameras).  
    /// The first [GLTFIdentifier] is for identifying the scene.  
    /// The second [GLTFIdentifier] is for identifying the actual resource.
    ///
    /// You can import only a given resource (e.g. models) by only filling out
    /// the corresponding map.
    LoadSpecific {
        scene_model_map: HashMap<GLTFIdentifier, Vec<GLTFIdentifier>>,
        scene_light_map: HashMap<GLTFIdentifier, Vec<GLTFIdentifier>>,
        scene_camera_map: HashMap<GLTFIdentifier, Vec<GLTFIdentifier>>,
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
    mode: Option<GLTFLoaderMode>,
    worker: Option<GLTFWorker>,
}

impl GLTFLoader {
    pub fn new(file_path: &'static str) -> Self
    where
        Self: Sized,
    {
        Self {
            file_path: file_path.into(),
            mode: None,
            worker: None,
        }
    }

    pub fn load_everything(&mut self) -> &mut Self {
        if self.mode.is_some() {
            warn!("GLTFLoaderMode already set, overwriting!");
        }

        self.mode = Some(GLTFLoaderMode::LoadEverything);

        self
    }

    pub fn load_scenes(&mut self, scenes: &mut Vec<GLTFIdentifier>) -> &mut Self {
        if let Some(mode) = &mut self.mode {
            match mode {
                GLTFLoaderMode::LoadScenes { identifiers } => {
                    // If the mode is already selected, add to it
                    identifiers.append(scenes);

                    return self;
                }
                _ => {
                    warn!("GLTFLoaderMode already set, overwriting!");
                }
            }
        }

        self.mode = Some(GLTFLoaderMode::LoadEverything);

        self
    }

    pub fn load_scene(&mut self, scene: GLTFIdentifier) -> &mut Self {
        self.load_scenes(&mut vec![scene])
    }

    pub fn load_model(&mut self, scene: GLTFIdentifier, model: GLTFIdentifier) -> &mut Self {
        if let Some(mode) = &mut self.mode {
            match mode {
                GLTFLoaderMode::LoadSpecific {
                    scene_model_map,
                    scene_light_map: _,
                    scene_camera_map: _,
                } => {
                    // If the mode is already selected, add to it
                    let entry = scene_model_map.entry(scene);

                    let models = entry.or_default();
                    models.push(model);

                    return self;
                }
                _ => {
                    warn!("GLTFLoaderMode already set, overwriting!");
                }
            }
        }

        let mut scene_model_map = HashMap::new();
        let scene_light_map = HashMap::new();
        let scene_camera_map = HashMap::new();

        scene_model_map.insert(scene, vec![model]);

        self.mode = Some(GLTFLoaderMode::LoadSpecific {
            scene_model_map,
            scene_light_map,
            scene_camera_map,
        });

        self
    }

    pub fn load_light(&mut self, scene: GLTFIdentifier, light: GLTFIdentifier) -> &mut Self {
        if let Some(mode) = &mut self.mode {
            match mode {
                GLTFLoaderMode::LoadSpecific {
                    scene_model_map: _,
                    scene_light_map,
                    scene_camera_map: _,
                } => {
                    // If the mode is already selected, add to it
                    let entry = scene_light_map.entry(scene);

                    let lights = entry.or_default();
                    lights.push(light);

                    return self;
                }
                _ => {
                    warn!("GLTFLoaderMode already set, overwriting!");
                }
            }
        }

        let scene_model_map = HashMap::new();
        let mut scene_light_map = HashMap::new();
        let scene_camera_map = HashMap::new();

        scene_light_map.insert(scene, vec![light]);

        self.mode = Some(GLTFLoaderMode::LoadSpecific {
            scene_model_map,
            scene_light_map,
            scene_camera_map,
        });

        self
    }

    pub fn load_camera(&mut self, scene: GLTFIdentifier, camera: GLTFIdentifier) -> &mut Self {
        if let Some(mode) = &mut self.mode {
            match mode {
                GLTFLoaderMode::LoadSpecific {
                    scene_model_map: _,
                    scene_light_map: _,
                    scene_camera_map,
                } => {
                    // If the mode is already selected, add to it
                    let entry = scene_camera_map.entry(scene);

                    let cameras = entry.or_default();
                    cameras.push(camera);

                    return self;
                }
                _ => {
                    warn!("GLTFLoaderMode already set, overwriting!");
                }
            }
        }

        let scene_model_map = HashMap::new();
        let scene_light_map = HashMap::new();
        let mut scene_camera_map = HashMap::new();

        scene_camera_map.insert(scene, vec![camera]);

        self.mode = Some(GLTFLoaderMode::LoadSpecific {
            scene_model_map,
            scene_light_map,
            scene_camera_map,
        });

        self
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

    fn loader_load_scenes(gltf_scenes: &[Scene], identifier: &GLTFIdentifier) -> Vec<WorldChange> {
        // TODO: Parallelise!

        match identifier {
            GLTFIdentifier::Id(ids) => gltf_scenes
                .into_iter()
                .enumerate()
                .filter_map(|(i, x)| ids.contains(&i).then_some(x))
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

        let file_path = self.file_path.clone();
        let mode = self.mode.clone().unwrap();

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
                GLTFLoaderMode::LoadScenes { identifiers } => {
                    let mut world_changes = Vec::new();

                    for identifier in identifiers {
                        let scene_world_change =
                            Self::loader_load_scenes(&gltf_scenes, &identifier);

                        world_changes.extend(scene_world_change);
                    }

                    world_changes
                }
                GLTFLoaderMode::LoadSpecific {
                    scene_model_map,
                    scene_light_map,
                    scene_camera_map,
                } => todo!(),
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
    let mut loader = GLTFLoader::new(TEST_FILE_PATH);

    loader.load_everything();

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
    let mut loader = GLTFLoader::new(TEST_FILE_PATH);

    loader.load_scene(GLTFIdentifier::Id(vec![0, 1, 2]));

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
    let mut loader = GLTFLoader::new(TEST_FILE_PATH);

    loader.load_scene(GLTFIdentifier::Label(vec!["Scene"]));

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
