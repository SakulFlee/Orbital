use std::{any::Any, fmt};

use hashbrown::HashMap;

use crate::{
    app::AppChange,
    game::Element,
    loader::Loader,
    resources::descriptors::{
        CameraDescriptor, LightDescriptor, MaterialDescriptor, ModelDescriptor,
    },
    variant::Variant,
};

use super::Identifier;

pub mod mode;
pub use mode::*;

pub mod camera;
pub use camera::*;

/// A [WorldChange] is a _proposed change_ to the [World].  
/// Basically, whenever we need to interact with the [World], it is through
/// a [WorldChange].
///
/// [World]: super::World
pub enum WorldChange {
    /// Spawns (== adds) an [Element] to the [World].
    ///
    /// ⚠️ The given [Element] must be [Boxed](Box) as it's a `dyn Trait`!
    ///
    /// [World]: super::World
    SpawnElement(Box<dyn Element>),
    /// Despawns (== removes) one or many [Element]s from the [World].
    ///
    /// If the identifier is used by multiple [Element]s, **all** will be
    /// despawned.
    ///
    /// [World]: super::World
    DespawnElement(Identifier),
    /// Spawns (== add) a [Model] into the [World].
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [World]: super::World
    SpawnModel(ModelDescriptor),
    /// Despawns (== removes) a [Model] from the [World].
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [World]: super::World
    DespawnModel(String),
    /// Sends a `Message` to one or many [Element]s.
    ///
    /// The message must be a [HashMap<String, Variant>] and can encode most
    /// information. Make sure to _"share information, not references"_!
    ///
    /// # Rejection
    /// Will be rejected if an [Element] with the specified _Identifier_ was
    /// not found.
    ///
    SendMessage(Identifier, HashMap<String, Variant>),
    /// Spawns a [Camera] into the [World].
    ///
    /// # Rejection
    /// Will be rejected if a [Camera]/[CameraDescriptor] with the specified
    /// _Identifier_ already exists.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    /// [World]: super::World
    SpawnCamera(CameraDescriptor),
    /// Does the same as [WorldChange::SpawnCamera], but also makes the new
    /// [Camera] become active.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    SpawnCameraAndMakeActive(CameraDescriptor),
    /// Despawns a [Camera] given a _Identifier_.
    ///
    /// # Rejection
    /// If a [Camera] with the given _identifier_ does not exist, the change
    /// will be rejected.
    ///
    /// # Active [Camera] removal
    /// If the **active** [Camera] is removed, a [Default] [Camera] will be
    /// spawned automatically on the next cycle.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    DespawnCamera(String),
    /// Changes the active [Camera] to a [Camera] with the
    /// given _identifier_.
    ///
    /// # Rejection
    /// If a [Camera] with the given _identifier_ does **not** exist,
    /// the change will be rejected.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    ChangeActiveCamera(String),
    /// Applies changes to the target [Camera].
    ///
    /// If a [Camera] exists with the specified [CameraDescriptor::identifier],
    /// any property that is set to `Some(...)` will be applied, and if the
    /// [Camera] is active, will trigger a [Buffer] update.  
    ///
    /// # Rejection
    /// If a [Camera] does not exists with the [CameraDescriptor::identifier],
    /// this world change will be rejected and a warning will be printed to
    /// console.
    ///
    /// [Camera]: crate::resources::realizations::Camera
    /// [Buffer]: wgpu::Buffer
    UpdateCamera(CameraChange),
    /// Passes on _any_ [AppChange]s that need to be processed
    /// by the [AppRuntime].
    ///
    /// [AppRuntime]: crate::app::AppRuntime
    AppChange(AppChange),
    /// Spawns a [Light] into the [World].
    ///
    /// [Light]: crate::resources::realizations::Light
    /// [World]: super::World
    SpawnLight(LightDescriptor),
    /// Changes the _World Environment_ for the [World].
    /// This is mainly changing the _SkyBox_ and _IBL_.
    ///
    /// [World]: super::World
    ChangeWorldEnvironment { skybox_material: MaterialDescriptor },
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
    /// Enqueues a [Loader] of any kind to the [World].
    /// The [World] will eventually start executing the [Loader] in the
    /// background and once done, process any [WorldChange]s proposed by it.
    ///
    /// [World]: super::World
    /// [Loader]: crate::loader::Loader
    EnqueueLoader(Box<dyn Loader + Send>),
}

impl fmt::Debug for WorldChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpawnElement(arg0) => write!(f, "SpawnElement@{:?}", arg0.type_id()),
            Self::DespawnElement(arg0) => f.debug_tuple("DespawnElement").field(arg0).finish(),
            Self::SpawnModel(arg0) => f.debug_tuple("SpawnModel").field(arg0).finish(),
            Self::DespawnModel(arg0) => f.debug_tuple("DespawnModel").field(arg0).finish(),
            Self::SendMessage(arg0, arg1) => f
                .debug_tuple("SendMessage")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::SpawnCamera(arg0) => f.debug_tuple("SpawnCamera").field(arg0).finish(),
            Self::SpawnCameraAndMakeActive(arg0) => f
                .debug_tuple("SpawnCameraAndMakeActive")
                .field(arg0)
                .finish(),
            Self::DespawnCamera(arg0) => f.debug_tuple("DespawnCamera").field(arg0).finish(),
            Self::ChangeActiveCamera(arg0) => {
                f.debug_tuple("ChangeActiveCamera").field(arg0).finish()
            }
            Self::UpdateCamera(arg0) => f.debug_tuple("UpdateCamera").field(arg0).finish(),
            Self::AppChange(app_change) => f.debug_tuple("AppChange").field(app_change).finish(),
            Self::SpawnLight(desc) => f.debug_tuple("SpawnLight").field(desc).finish(),
            WorldChange::ChangeWorldEnvironment { skybox_material } => f
                .debug_tuple("ChangeSkyBox")
                .field(skybox_material)
                .finish(),
            WorldChange::CleanWorld => f.debug_tuple("CleanWorld").finish(),
            WorldChange::EnqueueLoader(_) => f.debug_tuple("EnqueueLoader").finish(),
        }
    }
}
