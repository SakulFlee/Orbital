use std::{
    ops::Range,
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
    /// The numerical id, starting at zero (0), of the location of the model.
    /// glTF doesn't have an index system, meaning the 0th resource is always
    /// the first in list.
    /// The next element will always be the 1st, then 2nd, etc.
    /// Empty indices don't exist!
    Id(u32),
    /// Same as [GLTFIdentifier::Id], but with a range of Ids.
    Ids(Range<u32>),
    /// Names/Labels must be supported by the glTF file.
    /// It's an optional feature and is not always included!
    Name(&'static str),
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
    LoadScenes { identifier: Vec<GLTFIdentifier> },
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
                GLTFLoaderMode::LoadScenes { identifier } => {
                    // If the mode is already selected, add to it
                    identifier.append(scenes);

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

    fn loader_load_everything(gltf_scenes: &Vec<Scene>) -> Vec<WorldChange> {
        let mut world_changes = Vec::new();

        // TODO: Parallelise!

        for gltf_scene in gltf_scenes {
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
        }

        world_changes
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
                    sender.send(Err(Error::GltfError(e)));
                    return;
                }
            };

            let world_changes = match mode {
                GLTFLoaderMode::LoadEverything => Self::loader_load_everything(&gltf_scenes),
                GLTFLoaderMode::LoadScenes { identifier } => todo!(),
                GLTFLoaderMode::LoadSpecific {
                    scene_model_map,
                    scene_light_map,
                    scene_camera_map,
                } => todo!(),
            };

            sender.send(Ok(world_changes));
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

#[test]
fn test() {
    const FILE_PATH: &'static str = "../Assets/Models/PBR_Spheres.glb";
    let mut loader = GLTFLoader::new(FILE_PATH);

    loader.load_everything();

    println!("Begin processing glTF file: {}", FILE_PATH);
    let time = Instant::now();
    loader.begin_processing();

    while !loader.is_done_processing() {
        // thread::sleep(Duration::from_millis(10));
    }

    let result = loader.finish_processing();

    let elapsed = time.elapsed();
    println!("Took: {}ms", elapsed.as_millis());
    match result {
        Ok(v) => println!("Finished processing! {} WorldChange's generated!", v.len()),
        Err(e) => println!("Failed processing: {:?}", e),
    }
}
