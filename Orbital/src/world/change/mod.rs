use crate::{
    app::AppChange,
    world::Element,
    loader::Loader,
    resources::descriptors::{
        CameraDescriptor, LightDescriptor, MaterialDescriptor, ModelDescriptor, SkyboxType,
    },
    transform::Transform,
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
    /// Spawns (== adds) an [Element] to the [World].
    ///
    /// ⚠️ The given [Element] must be [Boxed](Box) as it's a `dyn Trait`!
    ///
    /// [World]: super::World
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
    /// [World]: super::World
    SpawnModel(ModelDescriptor),
    /// Despawns (== removes) a [Model] from the [World].
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [World]: super::World
    DespawnModel(String),
    /// If the given [Model] can be found, will _replace_ all given
    /// [Transform]s and set the given [Transform] as the only one. This should
    /// be used if you only have one [Instance]/[Transform] on your [Model].
    ///
    /// If multiple [Transform]s have been set before (i.e. the [Model] is
    /// [Instance]d), will remove any [Instance]s. Use the other [Transform]
    /// [WorldChange]s.
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [Instance]: crate::resources::realizations::Instance
    SetTransformModel(String, Transform),
    /// If the given [Model] can be found, will _replace_ a **specific**
    /// [Transform] by it's index. This should be used if you have multiple
    /// [Instance]s/[Transform]s on your [Model].
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [Instance]: crate::resources::realizations::Instance
    SetTransformSpecificModelInstance(String, Transform, usize),
    /// If the given [Model] can be found, will _apply_ a the given [Transform]
    /// to **all** defined [Transform]s. This should be used if you have
    /// multiple [Instance]s/[Transform]s on your [Model] and you want to
    /// offset them by a given [Transform]. This is especially useful if you
    /// have loaded e.g. a whole level and now want to move it by an offset.
    /// Applying here means, adding this [Transform] as an _offset_ to the
    /// existing [Transform] of the [Model].
    ///
    /// # Example:
    /// Given the following [Model] [Transform]:
    /// Transform::position at `(1, 2, 3)`
    ///
    /// ... and a Transform::position at `(5, 0, 0)`
    ///
    /// ... the result of this will be:
    /// ```
    /// (1 + 5, 2 + 0, 3 + 0)
    /// == (6, 2, 3).
    /// ```
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [Instance]: crate::resources::realizations::Instance
    ApplyTransformModel(String, Transform),
    /// If the given [Model] can be found, will _apply_ a **specific**
    /// [Transform] by it's index. This should be used if you have multiple
    /// [Instance]s/[Transform]s on your [Model].
    /// Applying here means, adding this [Transform] as an _offset_ to the
    /// existing [Transform] of the [Model].
    ///
    /// # Example:
    /// Given the following [Model] [Transform]:
    /// Transform::position at `(1, 2, 3)`
    ///
    /// ... and a Transform::position at `(5, 0, 0)`
    ///
    /// ... the result of this will be:
    /// ```
    /// (1 + 5, 2 + 0, 3 + 0)
    /// == (6, 2, 3).
    /// ```
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [Instance]: crate::resources::realizations::Instance
    ApplyTransformSpecificModelInstance(String, Transform, usize),
    /// If the given [Model] can be found, will add one or many [Transform]_s_
    /// to the [Model]. This will effectively **[Instance]** the [Model].
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [Instance]: crate::resources::realizations::Instance
    AddTransformsToModel(String, Vec<Transform>),
    /// If the given [Model] can be found, will remove one or many
    /// [Transform]_s_ from the [Model]. This will effectively **[Instance]**   
    /// the [Model].
    ///
    /// [Model]: crate::resources::realizations::Model
    /// [Instance]: crate::resources::realizations::Instance
    RemoveTransformsFromModel(String, Vec<usize>),
    /// Sends a `Message` to one or many [Element]s.
    ///
    /// The message must be a [Message] and can encode most
    /// information. Make sure to _"share information, not references"_!
    ///
    /// # Rejection
    /// Will be rejected if an [Element] with the specified _Identifier_ was
    /// not found.
    ///
    SendMessage(Message),
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
    /// Despawns a [Light] from the [World].
    ///
    /// [Light]: crate::resources::realizations::Light
    /// [World]: super::World
    DespawnLight(String),
    /// Changes the _World Environment_ for the [World].
    ///
    /// [World]: super::World
    ChangeWorldEnvironment {
        world_environment_material_descriptor: MaterialDescriptor,
    },
    /// Changes the [WorldEnvironment] [SkyboxType] for the [World].
    ///
    /// [World]: super::World
    ChangeWorldEnvironmentSkyboxType { skybox_type: SkyboxType },
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
    /// [World]: crate::world::World::World
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
