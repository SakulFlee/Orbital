use hashbrown::HashMap;
use ulid::Ulid;

use crate::{game::Element, resources::descriptors::ModelDescriptor, variant::Variant};

use super::{ElementUlid, ModelUlid};

pub enum WorldChange {
    SpawnElement(Box<dyn Element>),
    DespawnElement(ElementUlid),
    SpawnModel(ModelDescriptor),
    DespawnModel(ModelUlid),
    SendMessage(ElementUlid, HashMap<String, Variant>),
}
