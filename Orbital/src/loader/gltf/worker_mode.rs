use hashbrown::HashMap;

use super::GLTFIdentifier;

#[derive(Debug, Clone)]
pub enum GLTFWorkerMode {
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
