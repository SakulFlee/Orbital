use element_store::ElementStore;
use hashbrown::HashMap;
use log::{info, warn};
use wgpu::{Device, Queue};

use crate::{
    app::{AppChange, InputEvent},
    log::error,
    resources::{
        descriptors::{CameraDescriptor, MaterialDescriptor, ModelDescriptor},
        realizations::{Camera, Model},
    },
};

pub mod change;
pub use change::*;

pub mod element;
pub use element::*;

pub mod message;
pub use message::*;

pub mod loader_executor;
pub use loader_executor::*;

mod element_store;

mod light_store;
pub use light_store::*;

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
#[derive(Debug)]
pub struct World {
    element_store: ElementStore,
    /// **Active** [Model]s and their [Ulid]s
    models: HashMap<String, Model>,
    /// --- Storages ---
    light_store: LightStore,
    // --- Queues ---
    /// Queue for [WorldChange]s before being processed into other queues
    queue_world_changes: Vec<WorldChange>,
    /// Queue for spawning [Element]s
    queue_element_spawn: Vec<Box<dyn Element>>,
    /// Queue for despawning [Element]s
    queue_element_despawn: Vec<String>,
    /// Queue for spawning [Model]s
    queue_model_spawn: Vec<ModelDescriptor>,
    /// Queue for despawning [Model]s
    queue_model_despawn: Vec<String>,
    /// Queue for messages being send to a target [Ulid]
    queue_messages: HashMap<String, Vec<Message>>,
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
    // --- Environment ---
    world_environment: MaterialDescriptor,
    loader_executor: LoaderExecutor,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            element_store: ElementStore::new(),
            models: Default::default(),
            light_store: LightStore::new(),
            queue_world_changes: Default::default(),
            queue_element_spawn: Default::default(),
            queue_element_despawn: Default::default(),
            queue_model_spawn: Default::default(),
            queue_model_despawn: Default::default(),
            queue_messages: Default::default(),
            active_camera: Default::default(),
            active_camera_change: Default::default(),
            camera_descriptors: Default::default(),
            next_camera: Default::default(),
            world_environment: MaterialDescriptor::default_world_environment(),
            loader_executor: LoaderExecutor::new(None),
        }
    }

    fn process_active_camera_change(&mut self, device: &Device, queue: &Queue) {
        let update_option = self.active_camera_change.take();
        if let Some(change) = update_option {
            if let Some(camera) = &mut self.active_camera {
                camera.update_from_change(change, device, queue);
            } else {
                error!("Trying to apply camera change to active camera, but active camera does not exist!");
            }
        }
    }

    fn process_next_camera(&mut self, device: &Device, queue: &Queue) {
        if self.active_camera.is_none() && self.next_camera.is_none() {
            warn!("No active camera was set and no next camera is applied! Spawning a default camera ...");

            self.camera_descriptors.push(CameraDescriptor::default());
            self.next_camera = Some(CameraDescriptor::DEFAULT_NAME.into());
        }

        let taken = self.next_camera.take();
        if let Some(camera_identifier) = taken {
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
                }
            }
        }
    }

    fn process_queue_spawn_element(&mut self) {
        for mut element in self.queue_element_spawn.drain(..) {
            // Start element registration
            let registration: ElementRegistration = element.on_registration();

            let (labels, world_changes) = registration.extract();

            // Store boxed element
            self.element_store.store_element(element, labels);

            // Queue any changes
            self.queue_world_changes.extend(world_changes);
        }
    }

    fn process_queue_despawn_element(&mut self) {
        let drain = self.queue_element_despawn.drain(..).collect::<Vec<_>>();

        drain.iter().for_each(|element_label| {
            self.element_store.remove_element(element_label);
        });
    }

    fn process_queue_model_despawn(&mut self) {
        for model_ulid in self.queue_model_despawn.drain(..) {
            self.models.remove(&model_ulid);
        }
    }

    fn process_queue_model_spawn(&mut self) {
        for model_descriptor in self.queue_model_spawn.drain(..) {
            let model = Model::from_descriptor(model_descriptor);
            self.models.insert(model.label().to_string(), model);
        }
    }

    fn process_queue_messages(&mut self) {
        let mut world_changes = Vec::new();

        for (element_label, messages) in self.queue_messages.drain() {
            match self.element_store.send_messages(&element_label, messages) {
                Ok(element_world_changes) => {
                    world_changes.extend(element_world_changes);
                }
                Err(e) => {
                    error!(
                        "An error occurred while sending a message to '{}': {:?}",
                        element_label, e
                    )
                }
            }
        }

        for world_change in world_changes {
            self.process_world_change(world_change);
        }
    }

    pub fn process_world_changes(&mut self) -> Vec<AppChange> {
        let world_changes = std::mem::take(&mut self.queue_world_changes);

        let mut app_changes = Vec::new();
        for world_change in world_changes {
            if let Some(app_change) = self.process_world_change(world_change) {
                app_changes.push(app_change);
            }
        }

        self.process_queue_spawn_element();
        self.process_queue_despawn_element();
        self.process_queue_model_despawn();
        self.process_queue_messages();

        app_changes
    }

    /// Call this function to queue a given [WorldChange].  
    /// The [WorldChange] will be processed during the next possible
    /// cycle.
    pub fn process_world_change(&mut self, world_change: WorldChange) -> Option<AppChange> {
        match world_change {
            WorldChange::SpawnElement(element) => self.queue_element_spawn.push(element),
            WorldChange::DespawnElement(element_label) => {
                self.element_store.remove_element(&element_label);
            }
            WorldChange::SpawnModel(model_descriptor) => {
                self.queue_model_spawn.push(model_descriptor)
            }
            WorldChange::DespawnModel(model_label) => self.queue_model_despawn.push(model_label),
            WorldChange::SendMessage(element_label, message) => {
                match self
                    .element_store
                    .send_messages(&element_label, vec![message])
                {
                    Ok(world_changes) => self.queue_world_changes.extend(world_changes),
                    Err(e) => error!("An error occurred while sending a message: {:?}", e),
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
                        return None;
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
                } else if let Some(existing_camera_descriptor) = self
                    .camera_descriptors
                    .iter_mut()
                    .find(|x| x.identifier == change.target)
                {
                    existing_camera_descriptor.apply_change(change);
                }
            }
            WorldChange::AppChange(app_change) => return Some(app_change),
            WorldChange::SpawnLight(light_descriptor) => {
                self.light_store.add_light_descriptor(light_descriptor);
            }
            WorldChange::DespawnLight(label) => {
                self.light_store.remove_any_light_with_label(&label)
            }
            WorldChange::ChangeWorldEnvironment {
                skybox_material: world_environment_material_descriptor,
            } => {
                if let MaterialDescriptor::WorldEnvironment {
                    sky: _,
                    irradiance: _,
                    radiance: _,
                } = &world_environment_material_descriptor
                {
                    self.world_environment = world_environment_material_descriptor;
                } else {
                    error!("WorldChange::ChangeSkyBox requested, but supplied material is not of type MaterialDescriptor::SkyBox!");
                }
            }
            WorldChange::CleanWorld => {
                info!("WorldChange::CleanWorld received!");

                // Note: Materials and such will automatically clean up after the given cache interval is hit

                // Clear spawning queues
                self.queue_element_spawn.clear();
                self.queue_model_spawn.clear();

                // Elements
                self.element_store.clear();

                // Models
                self.models.clear();

                // Lights
                self.light_store.clear();

                // Camera
                self.camera_descriptors.clear();
                self.next_camera = None;
                self.active_camera = None;
                self.active_camera_change = None;
            }
            WorldChange::EnqueueLoader(loader) => {
                self.loader_executor.schedule_loader_boxed(loader);
            }
            WorldChange::ElementAddLabels {
                element_label,
                new_labels,
            } => {
                self.element_store.add_label(&element_label, new_labels);
            }
            WorldChange::ElementRemoveLabels {
                element_label,
                labels_to_be_removed,
            } => self
                .element_store
                .remove_label(&element_label, labels_to_be_removed),
            WorldChange::SetTransformModel(model_label, transform) => {
                if let Some(model) = self.models.get_mut(&model_label) {
                    model.set_transforms(vec![transform]);
                } else {
                    error!(
                        "Model with label '{}' could not be found! Cannot set transform: {:?}",
                        model_label, transform
                    );
                }
            }
            WorldChange::SetTransformSpecificModelInstance(model_label, transform, index) => {
                if let Some(model) = self.models.get_mut(&model_label) {
                    model.set_specific_transform(transform, index);
                } else {
                    error!(
                        "Model with label '{}' could not be found! Cannot set transform: {:?}",
                        model_label, transform
                    );
                }
            }
            WorldChange::ApplyTransformModel(model_label, transform) => {
                if let Some(model) = self.models.get_mut(&model_label) {
                    model.apply_transform(transform);
                } else {
                    error!(
                        "Model with label '{}' could not be found! Cannot apply transform: {:?}",
                        model_label, transform
                    );
                }
            }
            WorldChange::ApplyTransformSpecificModelInstance(model_label, transform, index) => {
                if let Some(model) = self.models.get_mut(&model_label) {
                    model.apply_transform_specific(transform, index);
                } else {
                    error!(
                        "Model with label '{}' could not be found! Cannot apply transform: {:?}",
                        model_label, transform
                    );
                }
            }
            WorldChange::AddTransformsToModel(model_label, transforms) => {
                if let Some(model) = self.models.get_mut(&model_label) {
                    model.add_transforms(transforms);
                } else {
                    error!(
                        "Model with label '{}' could not be found! Cannot add transforms: {:?}",
                        model_label, transforms
                    );
                }
            }
            WorldChange::RemoveTransformsFromModel(model_label, indices) => {
                if let Some(model) = self.models.get_mut(&model_label) {
                    model.remove_transforms(indices);
                } else {
                    error!("Model with label '{}' could not be found! Cannot remove transform at index '{:?}'", model_label, indices);
                }
            }
        }

        None
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

    pub fn on_focus_change(&mut self, focused: bool) {
        self.element_store.send_focus_change(focused);
    }

    pub fn on_input_event(&mut self, input_event: &InputEvent) {
        self.element_store.send_input_event(input_event);
    }

    /// Processes queued up [WorldChanges]
    ///
    /// ⚠️ This is already called automatically by the [GameRuntime].  
    /// ⚠️ You will only need to call this if you are making your own thing.
    ///
    /// [GameRuntime]: crate::game::GameRuntime
    pub fn update(&mut self, delta_time: f64) -> Vec<AppChange> {
        let element_changes = self.element_store.update(delta_time);
        self.queue_world_changes.extend(element_changes);

        // Cycle loader, enqueue any `Ok`, report any `Err`
        let (ok, error): (Vec<_>, Vec<_>) = self
            .loader_executor
            .cycle()
            .into_iter()
            .partition(|x| x.is_ok());

        self.queue_world_changes
            .extend(ok.into_iter().flat_map(|x| x.unwrap()).collect::<Vec<_>>());

        error
            .into_iter()
            .for_each(|x| error!("Failed loading resource with loader: {:?}", x.unwrap_err()));

        // Process through `WorldChange`s and pass on any `AppChange`s
        let mut app_changes = self.process_world_changes();
        if self.element_store.element_count() == 0 {
            warn!("No more elements in World! Exiting ...");

            app_changes.push(AppChange::ForceAppClosure { exit_code: 0 });
        }
        app_changes
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
        self.process_queue_model_spawn();
        self.process_active_camera_change(device, queue);
        self.process_next_camera(device, queue);

        self.light_store.update_if_needed(device, queue);

        self.models
            .values_mut()
            .for_each(|x| x.prepare_render(device, queue));
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

    pub fn active_camera(&self) -> &Camera {
        self.active_camera.as_ref().unwrap()
    }

    pub fn models(&self) -> Vec<&Model> {
        // TODO: Select models based on render radius
        self.models.values().by_ref().collect()
    }

    pub fn light_store(&self) -> &LightStore {
        &self.light_store
    }

    pub fn world_environment(&self) -> &MaterialDescriptor {
        &self.world_environment
    }
}
