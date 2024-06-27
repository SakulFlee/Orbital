use hashbrown::HashMap;
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
    queue_messages: HashMap<ElementUlid, HashMap<String, Variant>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_element<E: Element>(&mut self, element: E)
    where
        E: Sized + 'static,
    {
        // Box element for storage
        let mut boxed = Box::new(element);

        // Generate new ULID
        let ulid = Ulid::new();

        // Start element registration
        let registration = boxed.on_registration(&ulid);

        // Store boxed element
        self.elements.insert(ulid, boxed);

        // Process any tags
        if let Some(tags) = registration.tags {
            for tag in tags {
                self.tags
                    .entry(tag)
                    .or_insert(Vec::new())
                    .push(ulid.clone());
            }
        }

        // Process any models
        if let Some(models) = registration.models {
            for model in models {
                self.queue_model_spawn(model);
            }
        }
    }

    pub fn unregister_elements(&mut self, identifier: &Identifier) {
        // Convert identifier into list of ulids
        let ulids = match identifier {
            Identifier::Ulid(ulid) => vec![*ulid],
            Identifier::Tag(tag) => self
                .tag_to_ulids(tag)
                .map(|x| x.clone())
                .unwrap_or(Vec::new()),
        };

        // Queue despawn for any matches
        for ulid in ulids {
            self.queue_element_despawn.push(ulid);
        }
    }

    pub fn queue_model_spawn(&mut self, model_descriptor: ModelDescriptor) {
        self.queue_model_spawn.push((Ulid::new(), model_descriptor));
    }

    pub fn queue_model_despawn(&mut self, ulid: Ulid) {
        self.queue_model_despawn.push(ulid);
    }

    pub fn tag_to_ulids(&self, tag: &str) -> Option<&Vec<Ulid>> {
        self.tags.get(tag)
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
        for (element_id, message) in self.queue_messages.drain() {
            if let Some(element) = self.elements.get_mut(&element_id) {
                element.on_message(message);
            }
        }
    }

    pub(crate) fn update(&mut self, delta_time: f64) {
        for element in self.elements.values_mut() {
            if let Some(world_changes) = element.on_update(delta_time) {
                for world_change in world_changes {
                    match world_change {
                        WorldChange::SpawnElement(element) => {
                            self.queue_element_spawn.push(element)
                        }
                        WorldChange::DespawnElement(element_ulid) => todo!(),
                        WorldChange::SpawnModel(model_descriptor) => todo!(),
                        WorldChange::DespawnModel(model_ulid) => todo!(),
                        WorldChange::SendMessage(element_ulid, message) => todo!(),
                    }
                }
            }
        }

        self.process_queue_despawn_element();
        self.process_queue_model_despawn();
        self.process_queue_messages();
        // TODO: Spawn element queue

        // TODO: Remove queue functions
        // TODO: Make WorldChange processing function with above
    }

    pub fn prepare_render(&mut self, device: &Device, queue: &Queue) {
        self.process_queue_model_spawn(device, queue);
    }

    pub fn gather_models_to_render(&self) -> Vec<&Model> {
        self.models.values().collect::<Vec<_>>()
    }
}
