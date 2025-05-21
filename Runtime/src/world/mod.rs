use std::{future::Future, sync::Arc, time::Instant};

use async_std::sync::{Mutex, RwLock};
use futures::future::join_all;
use hashbrown::HashMap;
use log::{error, info, warn};
use wgpu::{Device, Queue};

use crate::{
    app::{input::InputState, AppChange},
    change_list::{ChangeList, ChangeListAction, ChangeListEntry, ChangeListType},
    element::{CameraChange, ElementChange, FileManager, Message, ModelChange, WorldChange},
    resources::{Camera, CameraDescriptor, ModelDescriptor, WorldEnvironmentDescriptor},
};

mod loader;
pub use loader::*;

pub type Store<T> = HashMap<String, Arc<RwLock<T>>>;
pub type ModelStore = Store<ModelDescriptor>;
pub type CameraStore = Store<CameraDescriptor>;

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
/// [Game]: crate::world::Game
/// [Elements]: crate::world::World::element::Element
/// [realized resource]: crate::resources::realizations
/// [realized resources]: crate::resources::realizations
#[derive(Debug)]
pub struct World
where
    Self: Send + Sync,
{
    element_store: RwLock<ElementStore>,
    model_store: RwLock<HashMap<String, Arc<RwLock<ModelDescriptor>>>>,
    camera_store: RwLock<HashMap<String, Arc<RwLock<CameraDescriptor>>>>,
    world_environment: WorldEnvironmentDescriptor,
    loader_executor: LoaderExecutor,
    close_requested_timer: Option<Instant>,
    world_change_queue: Vec<WorldChange>,
    change_list: Mutex<ChangeList>,
}

impl World {
    pub fn new(world_environment: WorldEnvironmentDescriptor) -> Self {
        Self {
            element_store: RwLock::new(ElementStore::new()),
            model_store: RwLock::new(HashMap::new()),
            camera_store: RwLock::new(HashMap::new()),
            world_change_queue: Default::default(),
            world_environment,
            loader_executor: LoaderExecutor::new(None),
            close_requested_timer: None,
            change_list: Mutex::new(ChangeList::new()),
        }
    }

    pub async fn process_world_changes(&mut self) -> Vec<AppChange> {
        let world_changes = std::mem::take(&mut self.world_change_queue);

        let futures = world_changes.into_iter().map(|x| self.process_world_change(x));

        let results = join_all(futures).await;
        let filtered_results = results.into_iter().filter_map(|x| x).collect();

        filtered_results
    }

    async fn process_model_change(&self, model_change: ModelChange) -> Option<AppChange> {
        match model_change {
            ModelChange::Spawn(model_descriptor) => {
                let mut model_store = self.model_store.write().await;

                let label = model_descriptor.label.clone();
                let arc = Arc::new(RwLock::new(model_descriptor));
                model_store.insert(label, arc);
            },
            ModelChange::Despawn(label) => {
                let mut model_store = self.model_store.write().await;

                if let None = model_store.remove(&label) {
                    warn!("Attempting to despawn non-existing model: {}!", label);
                }
            },
            ModelChange::Transform(label, mode) => {
                let model_store = self.model_store.read().await;

                match model_store.get(&label) {
                    Some(model) => {
                        let mut model_lock = model.write().await;

                        model_lock.apply_transform(mode);
                    }
                    None => {
                        error!("Attempting to transform non-existing Model with label '{}'! (Transform: {:?})", label, mode);
                    }
                }
            },
            ModelChange::TransformInstance(label, mode, index) => {
                let model_store = self.model_store.read().await;

                match model_store.get(&label) {
                    Some(model) => {
                        let mut model_lock = model.write().await;

                        model_lock.apply_transform_specific(mode, index);
                    }
                    None => {
                        error!("Attempting to transform non-existing Model with label '{}'! (Transform: {:?})", label, mode);
                    }
                }
            },
            ModelChange::AddInstance(label, transform) => {
                let model_store = self.model_store.read().await;

                match model_store.get(&label) {
                    Some(model) => {
                        let mut model_lock = model.write().await;

                        model_lock.add_transforms(vec![transform]);
                    },
                    None => {
                        error!("Attempting to add instance to non-existing Model with label '{}'! (Transform: {:?})", label, transform);
                    }
                }
            },
            ModelChange::RemoveInstance(label, index) => {
                let model_store = self.model_store.read().await;

                match model_store.get(&label) {
                    Some(model) => {
                        let mut model_lock = model.write().await;

                        model_lock.remove_transforms(vec![index]);
                    },
                    None => {
                        error!("Attempting to remove instance from non-existing Model with label '{}'! (Index: {:?})", label, index);
                    }
                }
            },
        }

        None // TODO
    }

    async fn process_camera_change(&self, camera_change: CameraChange) -> Option<AppChange> {
        match camera_change {
            CameraChange::Spawn(camera_descriptor) => {
                let mut  lock = self.camera_store.write().await;

                let label = camera_descriptor.label.clone();
                let arc = Arc::new(RwLock::new(camera_descriptor));
                lock.insert(label, arc);
            },           
            CameraChange::Despawn(label) => {
                let mut  lock = self.camera_store.write().await;

                lock.remove(&label);
            },
            CameraChange::Target(label) => {
                let mut  lock = self.camera_store.write().await;

                // TODO: Change for Renderer? Or pass on?
            },
            CameraChange::Transform(_, camera_transform) => todo!(),
        }

        None // TODO
    }

    async fn process_element_change(&self, element_change: ElementChange) -> Option<AppChange> {
        let lock = self.element_store.write().await;

        None // TODO
    }

    async fn process_app_change(&self, app_change: AppChange) -> Option<AppChange> {
        // let lock = self.app_store.write().await;

        None // TODO
    }

    async fn process_file_manager(&self, file_manager: FileManager) -> Option<AppChange> {
        // let lock = self.file_manager_store.write().await;

        None // TODO
    }

    async fn process_send_message(&self, message: Message) -> Option<AppChange> {
        // let lock = self.send_message.write().await;

        None // TODO
    }

    async fn process_clear(&self) -> Option<AppChange> {
        // let lock = self.app_store.write().await;

        None // TODO
    }

    async fn process_world_change(&self, world_change: WorldChange) -> Option<AppChange> {
        match world_change {
            WorldChange::Model(model_change) => self.process_model_change(model_change).await,
            WorldChange::Camera(camera_change) => self.process_camera_change(camera_change).await,
            WorldChange::Element(element_change) => self.process_element_change(element_change).await,
            WorldChange::App(app_change) => self.process_app_change(app_change).await,
            WorldChange::FileManager(file_manager) => self.process_file_manager(file_manager).await,
            WorldChange::SendMessage(message) => self.process_send_message(message).await,
            WorldChange::Clear => self.process_clear().await,
        }

        // WorldChange::SpawnElement(element) => {
        //     // Register Element
        //     let registration = element.on_registration();
        //     let (labels, world_changes) = registration.extract();

        //     // Enqueue new world changes
        //     self.world_change_queue.extend(world_changes);

        //     // Properly store Element after registration
        //     self.element_store.store_element(element, labels)
        // }
        // WorldChange::DespawnElement(element_label) => {
        //     self.element_store.remove_element(&element_label);
        // }
        // WorldChange::SpawnModel(model_descriptor) => {
        //     let x = Arc::new(RwLock::new(model_descriptor));

        //     self.change_list.lock().await.push(ChangeListEntry {
        //         change_type: ChangeListType::Model(x.clone()),
        //         action: ChangeListAction::Add,
        //     });

        //     self.model_store.insert(model_descriptor.label.clone(), x);
        // }
        // WorldChange::DespawnModel(model_label) => {
        //     if let Some(model) = self.model_store.remove(&model_label) {
        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Model(model),
        //             action: ChangeListAction::Remove,
        //         });
        //     }
        // }
        // WorldChange::SendMessage(message) => self.element_store.queue_message(message),
        // WorldChange::SendMessageToApp(message) => return Some(AppChange::SendMessage(message)),
        // WorldChange::SpawnCamera(descriptor) => {
        //     let mut descriptor = descriptor;
        //     if self.camera_store.is_empty() {
        //         descriptor.is_active = true;
        //     }

        //     let x = Arc::new(RwLock::new(descriptor));

        //     self.change_list.lock().await.push(ChangeListEntry {
        //         change_type: ChangeListType::Camera(x.clone()),
        //         action: ChangeListAction::Add,
        //     });

        //     self.spawn_camera(descriptor);
        // }
        // WorldChange::DespawnCamera(identifier) => {
        //     if let Some(camera) = self.camera_store.remove(&identifier) {
        //         let change_list_lock = self.change_list.lock().await;

        //         change_list_lock.push(ChangeListEntry {
        //             change_type: ChangeListType::Camera(camera),
        //             action: ChangeListAction::Remove,
        //         });

        //         // If we removed an *active* camera, we need to set the next one to be active. This will pick the next one in the list. This does not mean the exact next one in spawn order, but however this collection (map) is sorted.
        //         if self.camera_store.is_empty() && camera.read().await.is_active {
        //             let now_active_camera = self
        //                 .camera_store
        //                 .values_mut()
        //                 .take(1)
        //                 .map(|x| {
        //                     let y = x.get_mut();

        //                     // Set camera active
        //                     y.is_active = true;

        //                     // Clone the descriptor so we get a value and not a reference
        //                     x.clone()
        //                 })
        //                 .collect::<Vec<_>>()
        //                 // Removing here to "take" the camera from the store, instead of getting a reference
        //                 .remove(0);

        //             change_list_lock.push(ChangeListEntry {
        //                 change_type: ChangeListType::Camera(now_active_camera),
        //                 action: ChangeListAction::Change,
        //             });
        //         }
        //     } else {
        //         warn!("Attempting to despawn non-existing camera: {}!", identifier);
        //     }
        // }
        // WorldChange::MakeCameraActive(identifier) => {
        //     if let Some(descriptor) = self.camera_store.get_mut(&identifier) {
        //         descriptor.write().await.is_active = true;

        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Camera(descriptor.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     }
        // }
        // WorldChange::UpdateCamera(change) => {
        //     if let Some(descriptor) = self.camera_store.get_mut(&change.target) {
        //         descriptor.write().await.apply_change(change);
        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Camera(descriptor.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     } else {
        //         warn!("Attempting to change non-existing camera: {:?}!", change);
        //     }
        // }
        // WorldChange::AppChange(app_change) => return Some(app_change),
        // WorldChange::ChangeWorldEnvironment {
        //     world_environment_descriptor,
        // } => {
        //     self.world_environment = world_environment_descriptor;
        // }
        // WorldChange::CleanWorld => {
        //     info!("Cleaning world ...");

        //     // Elements
        //     self.element_store.clear();

        //     // Models
        //     self.model_store.clear();

        //     // Camera
        //     self.camera_store.clear();

        //     self.change_list.lock().await.push(ChangeListEntry {
        //         change_type: ChangeListType::All,
        //         action: ChangeListAction::Clear,
        //     });
        // }
        // WorldChange::ElementAddLabels {
        //     element_label,
        //     new_labels,
        // } => {
        //     self.element_store.add_label(&element_label, new_labels);
        // }
        // WorldChange::ElementRemoveLabels {
        //     element_label,
        //     labels_to_be_removed,
        // } => self
        //     .element_store
        //     .remove_label(&element_label, labels_to_be_removed),
        // WorldChange::TransformModel(model_label, transform) => {
        //     if let Some(model) = self.model_store.get(&model_label) {
        //         model.write().await.set_transforms(vec![transform]);

        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Model(model.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     } else {
        //         error!(
        //             "Model with label '{}' could not be found! Cannot set transform: {:?}",
        //             model_label, transform
        //         );
        //     }
        // }
        // WorldChange::ReplaceTransformSpecificModelInstance(model_label, transform, index) => {
        //     if let Some(model) = self.model_store.get(&model_label) {
        //         model.write().await.set_specific_transform(transform, index);

        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Model(model.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     } else {
        //         error!(
        //             "Model with label '{}' could not be found! Cannot set transform: {:?}",
        //             model_label, transform
        //         );
        //     }
        // }
        // WorldChange::ApplyTransformModel(model_label, transform) => {
        //     if let Some(model) = self.model_store.get(&model_label) {
        //         model.write().await.apply_transform(transform);

        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Model(model.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     } else {
        //         error!(
        //             "Model with label '{}' could not be found! Cannot apply transform: {:?}",
        //             model_label, transform
        //         );
        //     }
        // }
        // WorldChange::ApplyTransformSpecificModelInstance(model_label, transform, index) => {
        //     if let Some(model) = self.model_store.get(&model_label) {
        //         model
        //             .write()
        //             .await
        //             .apply_transform_specific(transform, index);

        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Model(model.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     } else {
        //         error!(
        //             "Model with label '{}' could not be found! Cannot apply transform: {:?}",
        //             model_label, transform
        //         );
        //     }
        // }
        // WorldChange::AddTransformsToModel(model_label, transforms) => {
        //     if let Some(model) = self.model_store.get(&model_label) {
        //         model.write().await.add_transforms(transforms);

        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Model(model.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     } else {
        //         error!(
        //             "Model with label '{}' could not be found! Cannot add transforms: {:?}",
        //             model_label, transforms
        //         );
        //     }
        // }
        // WorldChange::RemoveTransformsFromModel(model_label, indices) => {
        //     if let Some(model) = self.model_store.get(&model_label) {
        //         model.write().await.remove_transforms(indices);

        //         self.change_list.lock().await.push(ChangeListEntry {
        //             change_type: ChangeListType::Model(model.clone()),
        //             action: ChangeListAction::Change,
        //         });
        //     } else {
        //         error!(
        //             "Model with label '{}' could not be found! Cannot remove transform at index '{:?}'",
        //             model_label, indices
        //         );
        //     }
        // }
        // WorldChange::LoadFile { path } => {
        //     todo!()
        //     let gltf_loader = GLTFLoader::new(path, GLTFWorkerMode::LoadEverything, None);

        //     self.loader_executor
        //         .schedule_loader_boxed(Box::new(gltf_loader));
        // }
    }

    fn spawn_camera(&mut self, descriptor: CameraDescriptor) {
        if self
            .camera_descriptors
            .iter()
            .any(|x| x.label == descriptor.label)
        {
            warn!(
                "Trying to spawn Camera with identifier '{}', which already exists. Rejecting change!",
                descriptor.label
            );
            return;
        }

        self.camera_descriptors.push(descriptor);
    }

    /// Processes queued up [WorldChanges]
    ///
    /// ⚠️ This is already called automatically by the [GameRuntime].  
    /// ⚠️ You will only need to call this if you are making your own thing.
    ///
    /// [GameRuntime]: crate::world::GameRuntime
    pub async fn update(&mut self, delta_time: f64, input_state: &InputState) -> Vec<AppChange> {
        let element_changes = self.element_store.update(delta_time, input_state).await; todo!(): ?
        self.world_change_queue.extend(element_changes);

        // Cycle loader, enqueue any `Ok`, report any `Err`
        let (ok, error): (Vec<_>, Vec<_>) = self
            .loader_executor
            .cycle()
            .into_iter()
            .partition(|x| x.is_ok());

        self.world_change_queue
            .extend(ok.into_iter().flat_map(|x| x.unwrap()).collect::<Vec<_>>());

        error
            .into_iter()
            .for_each(|x| error!("Failed loading resource with loader: {:?}", x.unwrap_err()));

        // Process through `WorldChange`s and pass on any `AppChange`s
        let mut app_changes = self.process_world_changes().await;
        if app_changes.is_empty() && self.element_store.element_count() == 0 {
            match self.close_requested_timer {
                Some(timer) => {
                    // This should be after softly requesting!
                    if timer.elapsed().as_secs() > 5 {
                        warn!(
                            "No more elements in World and close request didn't work yet! Force Quitting!"
                        );
                        app_changes.push(AppChange::ForceAppClosure { exit_code: 0 });
                    }
                }
                None => {
                    // This should be first!
                    // Attempt to softly request the app to close first.
                    self.close_requested_timer = Some(Instant::now());
                    warn!("No more elements in World! Exiting ...");
                    app_changes.push(AppChange::RequestAppClosure);
                }
            }
        }
        app_changes
    }

    /// Similar to [World::update], but for [WorldChanges]
    /// that require GPU access.
    ///
    /// ⚠️ This is already called automatically by the [GameRuntime].  
    /// ⚠️ You will only need to call this if you are making your own thing.
    ///
    /// [GameRuntime]: crate::world::GameRuntime
    /// [WorldChanges]: WorldChange
    pub fn prepare_render(&mut self, device: &Device, queue: &Queue) {
        self.process_active_camera_change(device, queue);
        self.process_next_camera(device, queue);

        // self.light_store.update_if_needed(device, queue);
    }

    pub fn active_camera(&self) -> &Camera {
        self.active_camera.as_ref().unwrap()
    }

    pub fn element_store(&self) -> &ElementStore {
        &self.element_store
    }

    pub fn model_store(&self) -> &ModelStore {
        &self.model_store
    }

    // pub fn light_store(&self) -> &LightStore {
    //     &self.light_store
    // }

    pub fn world_environment(&self) -> &WorldEnvironmentDescriptor {
        &self.world_environment
    }

    /// **Takes** the current `ChangeList`, leaving behind an empty list.
    /// Should only be called once per frame by whoever needs it, most likely a `Renderer`!
    pub async fn take_change_list(&self) -> ChangeList {
        self.change_list.lock().await.drain(..).collect()
    }
}
