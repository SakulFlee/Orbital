use std::{any::Any, fmt};

use hashbrown::HashMap;

use crate::{
    app::AppChange,
    game::Element,
    loader::Loader,
    resources::descriptors::{
        CameraDescriptor, LightDescriptor, MaterialDescriptor, ModelDescriptor,
    },
    transform::Transform,
    variant::Variant,
};

use super::Identifier;

pub mod mode;
pub use mode::*;

pub mod camera;
pub use camera::*;

/// A [WorldChange] is a _proposed change to the [World]_.  
///
/// [World]: super::World
pub enum WorldChange {
    /// Queues an [Element] to be spawned.
    /// The given [Element] must be [Boxed](Box)!
    SpawnElement(Box<dyn Element>),
    /// Queues one or many [Element(s)](Element) to be despawned.
    /// Use an [Identifier] to select what to despawn!
    DespawnElement(Identifier),
    /// Queues a [Model] to be spawned.
    ///
    /// TODO: Update docs
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
    /// Sends a message to one or many [Elements](Element).  
    /// The message must be a [HashMap<String, Variant>].
    SendMessage(Identifier, HashMap<String, Variant>),
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
            Self::ChangeWorldEnvironment { skybox_material } => f
                .debug_tuple("ChangeSkyBox")
                .field(skybox_material)
                .finish(),
            Self::CleanWorld => f.debug_tuple("CleanWorld").finish(),
            Self::EnqueueLoader(_) => f.debug_tuple("EnqueueLoader").finish(),
            Self::SetTransformModel(model_label, transform) => f
                .debug_tuple("SetTransformModel")
                .field(model_label)
                .field(transform)
                .finish(),
            Self::SetTransformSpecificModelInstance(model_label, transform, index) => f
                .debug_tuple("SetTransformSpecificModelInstance")
                .field(model_label)
                .field(transform)
                .field(index)
                .finish(),
            Self::AddTransformsToModel(model_label, transforms) => f
                .debug_tuple("AddTransformsToModel")
                .field(model_label)
                .field(transforms)
                .finish(),
            Self::RemoveTransformsFromModel(model_label, indices) => f
                .debug_tuple("RemoveTransformsFromModel")
                .field(model_label)
                .field(indices)
                .finish(),
            Self::ApplyTransformModel(model_label, transform) => f
                .debug_tuple("ApplyTransformModel")
                .field(model_label)
                .field(transform)
                .finish(),
            Self::ApplyTransformSpecificModelInstance(model_label, transform, index) => f
                .debug_tuple("ApplyTransformSpecificModelInstance")
                .field(model_label)
                .field(transform)
                .field(index)
                .finish(),
        }
    }
}
