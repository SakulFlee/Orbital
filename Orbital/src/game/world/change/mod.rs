use std::{any::Any, fmt};

use gilrs::{ff::BaseEffect, GamepadId};
use hashbrown::HashMap;
use winit::{dpi::Position, window::Cursor};

use crate::{
    app::AppChange,
    game::Element,
    resources::descriptors::{CameraDescriptor, ModelDescriptor},
    variant::Variant,
};

use super::{ElementUlid, Identifier, ModelUlid};

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
    /// Same as [WorldChange::SpawnModel], but without needing to supply
    /// an [ElementUlid].
    /// The [ElementUlid] of the current [Element] will be used.
    ///
    /// [Model]: crate::resources::realizations::Model
    SpawnModelOwned(ModelDescriptor),
    /// Queues a [Model] to be spawned.
    ///
    /// Same as [WorldChange::SpawnModelOwned], but with needing to supply
    /// an [ElementUlid].
    ///
    /// [Model]: crate::resources::realizations::Model
    SpawnModel(ModelDescriptor, ElementUlid),
    /// Queues a [Model] to be despawned.  
    /// Use a [ModelUlid] to specify which is being despawned.
    ///
    /// [Model]: crate::resources::realizations::Model
    DespawnModel(ModelUlid),
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
    /// Changes the appearance (i.e. icon) of the mouse cursor.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorAppearance(Cursor),
    /// Changes the mouse cursor position.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorPosition(Position),
    /// Changes the mouse cursor visibility.  
    /// `true` means the cursor will be visible, whereas `false` means invisible.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorVisible(bool),
    /// Changes if the mouse cursor should be grabbed or not.  
    /// A grabbed mouse cursor **cannot** escape the current window.  
    /// Gets send directly to [winit], issues may appear in log.
    ChangeCursorGrabbed(bool),
    /// Used to activate vibration motors on a given gamepad.  
    /// Setting `gamepads` to `None` will result in **all** connected
    /// gamepads executing the given [BaseEffect].
    GamepadEffect {
        gamepads: Option<Vec<GamepadId>>,
        effects: Vec<BaseEffect>,
    },
}

impl From<WorldChange> for AppChange {
    fn from(value: WorldChange) -> Self {
        match value {
            WorldChange::ChangeCursorAppearance(x) => AppChange::ChangeCursorAppearance(x),
            WorldChange::ChangeCursorPosition(x) => AppChange::ChangeCursorPosition(x),
            WorldChange::ChangeCursorVisible(x) => AppChange::ChangeCursorVisible(x),
            WorldChange::ChangeCursorGrabbed(x) => AppChange::ChangeCursorGrabbed(x),
            _ => panic!(
                "Cannot convert {:?} to AppChange! Only select values are valid to be mapped.",
                value
            ),
        }
    }
}

impl fmt::Debug for WorldChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpawnElement(arg0) => write!(f, "SpawnElement@{:?}", arg0.type_id()),
            Self::DespawnElement(arg0) => f.debug_tuple("DespawnElement").field(arg0).finish(),
            Self::SpawnModelOwned(arg0) => f.debug_tuple("SpawnModelOwned").field(arg0).finish(),
            Self::SpawnModel(arg0, arg1) => {
                f.debug_tuple("SpawnModel").field(arg0).field(arg1).finish()
            }
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
            Self::ChangeCursorAppearance(arg0) => {
                f.debug_tuple("ChangeCursorAppearance").field(arg0).finish()
            }
            Self::ChangeCursorPosition(arg0) => {
                f.debug_tuple("ChangeCursorPosition").field(arg0).finish()
            }
            Self::ChangeCursorVisible(arg0) => {
                f.debug_tuple("ChangeCursorVisible").field(arg0).finish()
            }
            Self::ChangeCursorGrabbed(arg0) => {
                f.debug_tuple("ChangeCursorGrabbed").field(arg0).finish()
            }
            Self::GamepadEffect { gamepads, effects } => f
                .debug_tuple("GamepadEffect")
                .field(gamepads)
                .field(effects)
                .finish(),
        }
    }
}
