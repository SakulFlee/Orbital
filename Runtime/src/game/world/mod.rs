use hashbrown::HashMap;
use log::debug;
use ulid::Ulid;
use wgpu::{Device, Queue};

use crate::{
    log::error,
    resources::{descriptors::ModelDescriptor, realizations::Model},
    variant::Variant,
};

pub mod change;
pub use change::*;

pub mod element;
pub use element::*;

pub mod identifier;
pub use identifier::*;

pub type ElementUlid = Ulid;
pub type ModelUlid = Ulid;

#[derive(Default)]
pub struct World {
    /// [Element]s and their [Ulid]s
    elements: HashMap<ElementUlid, Box<dyn Element>>,
    /// **Active** [Model]s and their [Ulid]s
    models: HashMap<ModelUlid, Model>,
    /// Translation map to determine ownership over [Model]s
    /// based on [Element] [Ulid]s
    model_owner: HashMap<ModelUlid, ElementUlid>,
    /// Translation map to determine _tag_ association between [Element]s
    tags: HashMap<String, Vec<ElementUlid>>,
    /// Queue for spawning [Element]s
    queue_element_spawn: Vec<Box<dyn Element>>,
    /// Queue for despawning [Element]s
    queue_element_despawn: Vec<ElementUlid>,
    /// Queue for spawning [Model]s
    queue_model_spawn: Vec<(ElementUlid, ModelDescriptor)>,
    /// Queue for despawning [Model]s
    queue_model_despawn: Vec<ModelUlid>,
    /// Queue for messages being send to a target [Ulid]
    queue_messages: HashMap<ElementUlid, Vec<HashMap<String, Variant>>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    fn process_queue_spawn_element(&mut self) {
        let mut to_be_added_to_model_queue = Vec::new();

        for mut element in self.queue_element_spawn.drain(..) {
            // Generate new ULID
            let element_ulid = Ulid::new();

            debug!("New element: {}", element_ulid);

            // Start element registration
            let registration = element.on_registration(&element_ulid);

            // Store boxed element
            self.elements.insert(element_ulid, element);

            // Process any tags
            if let Some(tags) = registration.tags {
                for tag in tags {
                    self.tags
                        .entry(tag)
                        .or_insert(Vec::new())
                        .push(element_ulid.clone());
                }
            }

            // Process any models
            if let Some(models) = registration.models {
                for model in models {
                    to_be_added_to_model_queue.push((element_ulid, model));
                }
            }
        }

        for tuple in to_be_added_to_model_queue {
            self.queue_model_spawn.push(tuple);
        }
    }

    fn process_queue_despawn_element(&mut self) {
        let drain = self.queue_element_despawn.drain(..).collect::<Vec<_>>();

        drain.iter().for_each(|element_ulid| {
            // Remove the element
            self.elements.remove(element_ulid);

            // Find any ModelUlid and queue those for removal
            self.model_owner
                .iter()
                .filter(|(_, v)| *v == element_ulid)
                .map(|(k, _)| k)
                .for_each(|x| {
                    self.queue_model_despawn.push(*x);
                });
        });
    }

    fn process_queue_model_despawn(&mut self) {
        for model_ulid in self.queue_model_despawn.drain(..) {
            self.models.remove(&model_ulid);
        }
    }

    fn process_queue_model_spawn(&mut self, device: &Device, queue: &Queue) {
        for (element_id, model_descriptor) in self.queue_model_spawn.drain(..) {
            let model = match Model::from_descriptor(&model_descriptor, device, queue) {
                Ok(model) => model,
                Err(e) => {
                    error!(
                        "Failure realizing model for element '{}': {:#?}",
                        element_id, e
                    );
                    continue;
                }
            };

            let model_id = Ulid::new();
            self.models.insert(model_id, model);
            self.model_owner.insert(model_id, element_id);
        }
    }

    fn process_queue_messages(&mut self) {
        for (element_id, messages) in self.queue_messages.drain() {
            if let Some(element) = self.elements.get_mut(&element_id) {
                for message in messages {
                    element.on_message(message);
                }
            }
        }
    }

    pub fn queue_world_change(&mut self, world_change: WorldChange) {
        match world_change {
            WorldChange::SpawnElement(element) => self.queue_element_spawn.push(element),
            WorldChange::DespawnElement(element_ulid) => {
                self.queue_element_despawn.push(element_ulid)
            }
            WorldChange::SpawnModel(model_descriptor, element_ulid) => self
                .queue_model_spawn
                .push((element_ulid, model_descriptor)),
            WorldChange::SpawnModelOwned(_) => {
                error!("SpawnModelOwned cannot be used directly. Use SpawnModel instead!");
            }
            WorldChange::DespawnModel(model_ulid) => self.queue_model_despawn.push(model_ulid),
            WorldChange::SendMessage(identifier, message) => {
                for element_ulid in self.resolve_identifier(identifier) {
                    self.queue_messages
                        .entry(element_ulid)
                        .or_insert(Vec::new())
                        .push(message.clone());
                }
            }
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        let mut world_changes = Vec::new();

        for (element_ulid, element) in &mut self.elements {
            if let Some(element_world_changes) = element.on_update(delta_time) {
                for element_world_change in element_world_changes {
                    // Convert owned model spawning to auto-include the [ElementUlid]
                    if let WorldChange::SpawnModelOwned(x) = element_world_change {
                        world_changes.push(WorldChange::SpawnModel(x, *element_ulid));
                    } else {
                        world_changes.push(element_world_change);
                    }
                }
            }
        }

        for world_change in world_changes {
            self.queue_world_change(world_change);
        }

        self.process_queue_spawn_element();
        self.process_queue_despawn_element();
        self.process_queue_model_despawn();
        self.process_queue_messages();
    }

    pub fn prepare_render(&mut self, device: &Device, queue: &Queue) {
        self.process_queue_model_spawn(device, queue);
    }

    pub fn gather_models_to_render(&self) -> Vec<&Model> {
        self.models.values().collect::<Vec<_>>()
    }

    pub fn tag_to_ulids(&self, tag: &str) -> Option<&Vec<Ulid>> {
        self.tags.get(tag)
    }

    pub fn resolve_identifier(&self, identifier: Identifier) -> Vec<Ulid> {
        let mut ulids = Vec::new();

        match identifier {
            Identifier::Ulid(ulid) => ulids.push(ulid),
            Identifier::Tag(tag) => {
                if let Some(tag_ulids) = self.tag_to_ulids(&tag) {
                    for tag_ulid in tag_ulids {
                        ulids.push(*tag_ulid);
                    }
                }
            }
        }

        ulids
    }
}
