use hashbrown::HashMap;

use crate::{game::Element, resources::descriptors::ModelDescriptor, variant::Variant};

use super::{ElementUlid, Identifier, ModelUlid};

pub enum WorldChange {
    SpawnElement(Box<dyn Element>),
    DespawnElement(ElementUlid),
    /// Queues a model to be spawned
    ///
    /// Same as [Self::SpawnModel], but without needing to supply
    /// an [ElementUlid].
    /// The [ElementUlid] of the current [Element] will be used.
    SpawnModelOwned(ModelDescriptor),
    /// Queues a model to be spawned
    ///
    /// Same as [Self::SpawnModelOwned], but with needing to supply
    /// an [ElementUlid].
    SpawnModel(ModelDescriptor, ElementUlid),
    DespawnModel(ModelUlid),
    SendMessage(Identifier, HashMap<String, Variant>),
}
