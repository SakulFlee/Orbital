use std::any::Any;

use hashbrown::HashMap;
use log::{debug, warn};
use ulid::Ulid;
use wgpu::{Device, Queue};

use crate::{
    app::{AppChange, InputEvent},
    log::error,
    resources::{
        descriptors::{CameraDescriptor, ModelDescriptor},
        realizations::{Camera, Model},
    },
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

/// A [World] keeps track of everything inside your [Game].  
/// Mainly, [Elements] and [realized resources].
///
/// You may also be interested in [WorldChange] and [Element].
///
/// The inner workings of this structure are quiet complex, but do not
/// need to be understood by the average user.  
/// To break down what is happening here:  
/// - Each [Element] and [realized resource] gets assigned a [Ulid].
///     - Said [Ulid] is used as an identifier.
/// - When an [Element] spawns a [Model], the [realized resource] gets it's
///     [Ulid] assigned and a translation map is filled with both the [Element]
///     [Ulid], as well as the [realized resource] [Ulid].
/// - If a [realized resource] is despawned, it can be done so via their [Ulid].
/// - If an [Element] is despawned, it can be done so via their [Ulid].
///     - If this happens, any relations to [realized resources] **will also be
///         removed**.
///
/// Additionally, a _tagging_ system is in place here.  
/// If an [Element] registers itself with a given _tag_, then the _tag_ can be
/// used to interact with said [Element].
/// If multiple [Elements] register the same _tag_, then **all** will be
/// interacted with.  
/// E.g. You have three [Elements] _tagged_ `enemy` and request a despawning
/// of said _tag_ `enemy`, will result in **all** [Elements] _tagged_ `enemy`
/// to be removed.
///
/// Lastly, any changes and messaging is **queued up** and will be processed
/// on the next cycle if possible.
/// Changes get executed in-order of queueing (FIFO).
///
/// [Game]: crate::game::Game
/// [Elements]: crate::game::world::element::Element
/// [realized resource]: crate::resources::realizations
/// [realized resources]: crate::resources::realizations
#[derive(Default)]
pub struct World {
    // --- Elements & Models ---
    /// [Element]s and their [Ulid]s
    elements: HashMap<ElementUlid, Box<dyn Element>>,
    /// **Active** [Model]s and their [Ulid]s
    models: HashMap<ModelUlid, Model>,
    /// Translation map to determine ownership over [Model]s
    /// based on [Element] [Ulid]s
    model_owner: HashMap<ModelUlid, ElementUlid>,
    /// Translation map to determine _tag_ association between [Element]s
    tags: HashMap<String, Vec<ElementUlid>>,
    // --- Queues ---
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
    // --- Camera ---
    /// Active Camera
    active_camera: Option<Camera>,
    /// Active Camera Update  
    /// In case the camera to be updated is the active one.
    active_camera_change: Option<CameraChange>,
    /// Cameras
    camera_descriptors: Vec<CameraDescriptor>,
    /// Next camera to be changed to upon next cycle.
    /// Must be set to `Some` if we do change.
    /// Must be set to `None` if we don't change.
    /// Internal of `Some` must match existing camera descriptor.
    ///
    /// ⚠️ Only the most recent `WorldChange` request will be applied as we can
    /// only ever have one single camera active!
    next_camera: Option<String>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    fn process_active_camera_change(&mut self, device: &Device, queue: &Queue) {
        let update_option = self.active_camera_change.take();
        match update_option {
            Some(change) => {
                if let Some(camera) = &mut self.active_camera {
                    camera.update_from_change(change, device, queue);
                } else {
                    error!("Trying to apply camera change to active camera, but active camera does not exist!");
                }
            }
            None => return,
        }
    }

    fn process_next_camera(&mut self, device: &Device, queue: &Queue) {
        if self.active_camera.is_none() && self.next_camera.is_none() {
            warn!("No active camera was set and no next camera is applied! Spawning a default camera ...");

            self.camera_descriptors.push(CameraDescriptor::default());
            self.next_camera = Some(CameraDescriptor::DEFAULT_NAME.into());
        }

        let taken = self.next_camera.take();
        match taken {
            Some(camera_identifier) => {
                match self
                    .camera_descriptors
                    .iter()
                    .find(|x| x.identifier == camera_identifier)
                {
                    Some(camera_descriptor) => {
                        // Realize camera
                        self.active_camera = Some(Camera::from_descriptor(
                            camera_descriptor.clone(),
                            device,
                            queue,
                        ));
                    }
                    None => {
                        error!(
                            "Supposed to change to camera '{}', but no such camera exists!",
                            camera_identifier
                        );
                        return;
                    }
                }
            }
            None => return,
        }
    }

    fn process_queue_spawn_element(&mut self) {
        let mut world_changes_to_queue = Vec::new();
        let mut model_spawns_to_queue = Vec::new();

        for mut element in self.queue_element_spawn.drain(..) {
            // Generate new ULID
            let element_ulid = Ulid::new();
            debug!("New element: {}@{:?}", element_ulid, element.type_id());

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
                    model_spawns_to_queue.push((element_ulid, model));
                }
            }

            // Queue any changes
            if let Some(element_world_changes) = registration.world_changes {
                for world_change in element_world_changes {
                    world_changes_to_queue.push(world_change);
                }
            }
        }

        // Queue any model spawns
        for tuple in model_spawns_to_queue {
            self.queue_model_spawn.push(tuple);
        }

        // Queue any world changes
        for world_change in world_changes_to_queue {
            self.queue_world_change(world_change);
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
        let mut world_changes = Vec::new();

        for (element_id, messages) in self.queue_messages.drain() {
            if let Some(element) = self.elements.get_mut(&element_id) {
                for message in messages {
                    let result = element.on_message(message);

                    if let Some(result_world_changes) = result {
                        world_changes.extend(result_world_changes);
                    }
                }
            }
        }

        for world_change in world_changes {
            self.queue_world_change(world_change);
        }
    }

    /// Call this function to queue a given [WorldChange].  
    /// The [WorldChange] will be processed during the next possible
    /// cycle.
    pub fn queue_world_change(&mut self, world_change: WorldChange) {
        match world_change {
            WorldChange::SpawnElement(element) => self.queue_element_spawn.push(element),
            WorldChange::DespawnElement(identifier) => {
                for element_ulid in self.resolve_identifier(identifier) {
                    self.queue_element_despawn.push(element_ulid)
                }
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
            WorldChange::SpawnCamera(descriptor) => self.spawn_camera(descriptor),
            WorldChange::SpawnCameraAndMakeActive(descriptor) => {
                let identifier = descriptor.identifier.clone();
                self.spawn_camera(descriptor);
                self.next_camera = Some(identifier);
            }
            WorldChange::DespawnCamera(identifier) => {
                if let Some(camera) = &self.active_camera {
                    if camera.descriptor().identifier == identifier {
                        self.active_camera = None;

                        warn!("Despawned Camera was active!");
                    }
                }

                self.camera_descriptors
                    .retain(|x| x.identifier != identifier);
            }
            WorldChange::ChangeActiveCamera(identifier) => {
                if let Some(camera) = &self.active_camera {
                    if camera.descriptor().identifier == identifier {
                        warn!("Attempting to activate already active camera!");
                        return;
                    }
                }

                // If it exists or not will be handled by queue processor
                self.next_camera = Some(identifier);
            }
            WorldChange::UpdateCamera(change) => {
                if let Some(camera) = &self.active_camera {
                    if camera.descriptor().identifier == change.target {
                        self.active_camera_change = Some(change);
                    }
                } else {
                    if let Some(existing_camera_descriptor) = self
                        .camera_descriptors
                        .iter_mut()
                        .find(|x| x.identifier == change.target)
                    {
                        existing_camera_descriptor.apply_change(change);
                    }
                }
            }
            _ => (),
        }
    }

    fn spawn_camera(&mut self, descriptor: CameraDescriptor) {
        if self
            .camera_descriptors
            .iter()
            .any(|x| x.identifier == descriptor.identifier)
        {
            warn!("Trying to spawn Camera with identifier '{}', which already exists. Rejecting change!", descriptor.identifier);
            return;
        }

        self.camera_descriptors.push(descriptor);
    }

    pub fn on_input_event(&mut self, delta_time: f64, input_event: &InputEvent) {
        for (_element_ulid, element) in &mut self.elements {
            element.on_input_event(delta_time, input_event)
        }
    }

    /// Processes queued up [WorldChanges]
    ///
    /// ⚠️ This is already called automatically by the [GameRuntime].  
    /// ⚠️ You will only need to call this if you are making your own thing.
    ///
    /// [GameRuntime]: crate::game::GameRuntime
    pub fn update(&mut self, delta_time: f64) -> Option<Vec<AppChange>> {
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

        let (to_be_returned, queue_for_world): (Vec<_>, Vec<_>) =
            world_changes.into_iter().partition(|x| match x {
                WorldChange::ChangeCursorAppearance(_) => true,
                WorldChange::ChangeCursorPosition(_) => true,
                WorldChange::ChangeCursorVisible(_) => true,
                WorldChange::ChangeCursorGrabbed(_) => true,
                WorldChange::GamepadEffect {
                    gamepads: _,
                    effects: _,
                } => true,
                _ => false,
            });

        for world_change in queue_for_world {
            self.queue_world_change(world_change);
        }

        self.process_queue_spawn_element();
        self.process_queue_despawn_element();
        self.process_queue_model_despawn();
        self.process_queue_messages();

        if to_be_returned.is_empty() {
            None
        } else {
            Some(to_be_returned.into_iter().map(|x| x.into()).collect())
        }
    }

    /// Similar to [World::update], but for [WorldChanges]
    /// that require GPU access.
    ///
    /// ⚠️ This is already called automatically by the [GameRuntime].  
    /// ⚠️ You will only need to call this if you are making your own thing.
    ///
    /// [GameRuntime]: crate::game::GameRuntime
    /// [WorldChanges]: WorldChange
    pub fn prepare_render(&mut self, device: &Device, queue: &Queue) {
        self.process_queue_model_spawn(device, queue);
        self.process_active_camera_change(device, queue);
        self.process_next_camera(device, queue);
    }

    /// This function returns a [Vec<&Model>] of all [Models] that
    /// need to be rendered.
    /// This information is intended to be send to a [Renderer].
    ///
    /// [Models]: Model
    /// [Renderer]: crate::renderer::Renderer
    pub fn gather_render_resources(&self) -> (&Camera, Vec<&Model>) {
        (
            self.active_camera.as_ref().unwrap(),
            self.models.values().collect::<Vec<_>>(),
        )
    }

    /// Converts a given `tag` into a [Ulid] if found.
    pub fn tag_to_ulids(&self, tag: &str) -> Option<&Vec<Ulid>> {
        self.tags.get(tag)
    }

    /// Converts a given [Identifier] into a [Vec<Ulid>].
    /// I.e. a list of [Element] [Ulids].
    ///
    /// This is especially useful for resolving _tags_ as multiple
    /// [Elements] can be _tagged_ with the same _tag_.
    ///
    /// [Ulids]: Ulid
    /// [Elements]: Element
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

    // pub fn handle_input_event(&mut self, input_event: InputEvent) -> Result<(), Error> {
    //     // self.input_manager.handle_input_event(input_event)
    // }
}
