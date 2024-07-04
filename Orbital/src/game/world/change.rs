use std::{any::Any, fmt};

use hashbrown::HashMap;

use crate::{game::Element, resources::descriptors::ModelDescriptor, variant::Variant};

use super::{ElementUlid, Identifier, ModelUlid};

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
        }
    }
}
