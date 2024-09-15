use crate::{
    app::AppChange,
    game::Element,
    loader::Loader,
    resources::descriptors::{
        CameraDescriptor, LightDescriptor, MaterialDescriptor, ModelDescriptor,
    },
};

pub mod mode;
pub use mode::*;

pub mod camera;
pub use camera::*;

use super::Message;

/// A [WorldChange] is a _proposed change to the [World]_.  
///
/// [World]: super::World
#[derive(Debug)]
pub enum WorldChange {
    /// Queues an [Element] to be spawned.
    /// The given [Element] must be [Boxed](Box)!
    SpawnElement(Box<dyn Element>),
    /// Queues one or many [Element(s)](Element) to be despawned.
    DespawnElement(String),
    ElementAddLabels {
        element_label: String,
        new_labels: Vec<String>,
    },
    ElementRemoveLabels {
        element_label: String,
        labels_to_be_removed: Vec<String>,
    },
    /// Queues a [Model] to be spawned.
    ///
    /// Same as [WorldChange::SpawnModel], but without needing to supply
    /// an [ElementUlid].
    /// The [ElementUlid] of the current [Element] will be used.
    ///
    /// [Model]: crate::resources::realizations::Model
    SpawnModel(ModelDescriptor),
    /// Queues a [Model] to be despawned.  
    /// Use a [ModelUlid] to specify which is being despawned.
    ///
    /// TODO: Update docs
    ///
    /// [Model]: crate::resources::realizations::Model
    DespawnModel(String),
    /// Sends a message to one or many [Elements](Element).  
    /// The message must be of type [Message], which is an alias for
    /// [HashMap<String, Variant>].
    SendMessage(String, Message),
    /// Spawns a [Camera] with a given [CameraDescriptor].  
    /// If the chosen `identifier` of the [Camera] is already taken, this change
    /// will be rejected.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    SpawnCamera(CameraDescriptor),
    /// Similar to [Self::SpawnCamera], but also makes the new [Camera] active.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    SpawnCameraAndMakeActive(CameraDescriptor),
    /// Despawns a [Camera] given a `identifier` ([String]).
    ///
    /// If a [Camera] with the given `identifier` exists, it will be removed.  
    /// If a [Camera] with the given `identifier` does not exist, nothing will happen.
    ///
    /// If the active [Camera] is removed, a [Default] [Camera] will be spawned
    /// automatically on the next cycle.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    DespawnCamera(String),
    /// Changes the active [Camera] to a [Camera] with the
    /// given `identifier` ([String]).
    ///
    /// If a [Camera] with the given `identifier` does **not** exist,
    /// the active [Camera] **will not be changed**.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    ChangeActiveCamera(String),
    /// Applies changes to the target [Camera].
    ///
    /// If a [Camera] exists with the specified [CameraDescriptor::identifier],
    /// any property that is set to `Some(...)` will be applied, and if the
    /// [Camera] is active, will trigger a [Buffer] update.  
    /// If a [Camera] does not exists with the [CameraDescriptor::identifier],
    /// this world change will be rejected and a warning will be printed to
    /// console.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    /// [Buffer]: wgpu::Buffer
    UpdateCamera(CameraChange),
    /// Any [AppChange]s that need to be processed need to use this variant!
    AppChange(AppChange),
    /// Spawns a light into existence.
    SpawnLight(LightDescriptor),
    ChangeWorldEnvironment {
        skybox_material: MaterialDescriptor,
    },
    /// Cleans the entire [World].
    /// Meaning, that any [Element]s, and their associated resources like
    /// [Model]s, will be despawned and
    /// removed from the world.
    ///
    /// This can be used to replicate the effect of a "scene change".  
    /// Queue a [WorldChange::CleanWorld] first, then anything for the new
    /// scene.
    ///
    /// ⚠️This will literally remove everything from the [World].
    /// Be careful!⚠️  
    /// The order of enqueueing [WorldChange]s matters here!  
    /// For example:  
    /// Say, we queue in the following order:  
    /// 1. [WorldChange::SpawnElement]  
    /// 2. [WorldChange::CleanWorld]  
    /// 3. [WorldChange::SpawnElement]  
    /// We would only end up with **ONE** spawned [Element].
    /// The first one would be nulled.
    ///
    ///
    /// [World]: crate::game::world::World
    /// [Model]: crate::resources::realizations::Model
    CleanWorld,
    EnqueueLoader(Box<dyn Loader + Send>),
}
