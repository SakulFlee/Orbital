use std::sync::Arc;

use async_std::sync::RwLock;
use log::warn;

use crate::element::{CameraEvent, ModelEvent};

mod store;
pub use store::*;

mod event;
pub use event::*;

mod change_list;
pub use change_list::*;

pub struct Physics {
    model_store: ModelStore,
    camera_store: CameraStore,
}

impl Physics {
    pub fn new() -> Self {
        // TODO: implement a real physics system
        warn!("JUST A DUMMY PLACEHOLDER PHYSICS SYSTEM FOR NOW!");

        Self {
            model_store: ModelStore::new(),
            camera_store: CameraStore::new(),
        }
    }

    pub async fn update(
        &mut self,
        _delta_time: f64,
        events: Vec<PhysicsEvent>,
    ) -> Option<ChangeList> {
        let mut change_list = ChangeList::new();

        for event in events {
            let change_list_entry_option = match event {
                PhysicsEvent::Model(model_event) => self.handle_model_event(model_event).await,
                PhysicsEvent::Camera(camera_event) => self.handle_camera_event(camera_event).await,
                PhysicsEvent::Clear => self.handle_clear_event().await,
            };

            if let Some(change_list_entry) = change_list_entry_option {
                change_list.push(change_list_entry);
            }
        }

        (!change_list.is_empty()).then_some(change_list)
    }

    async fn handle_model_event(&mut self, event: ModelEvent) -> Option<ChangeListEntry> {
        match event {
            ModelEvent::Spawn(model_descriptor) => {
                let label = model_descriptor.label.clone();
                let value = Arc::new(RwLock::new(model_descriptor));

                self.model_store.insert(label, value.clone());

                Some(ChangeListEntry::Model(ModelChangeListEntry::Spawn(value)))
            }
            ModelEvent::Despawn(label) => {
                if let Some(arc) = self.model_store.remove(&label) {
                    Some(ChangeListEntry::Model(ModelChangeListEntry::Despawn(arc)))
                } else {
                    None
                }
            }
            ModelEvent::Transform(label, mode) => {
                if let Some(arc) = self.model_store.get(&label) {
                    arc.write().await.apply_transform(mode);

                    Some(ChangeListEntry::Model(ModelChangeListEntry::Change(*arc)))
                } else {
                    None
                }
            }
            ModelEvent::TransformInstance(label, mode, idx) => {
                if let Some(arc) = self.model_store.get(&label) {
                    arc.write().await.apply_transform_specific(mode, idx);

                    Some(ChangeListEntry::Model(ModelChangeListEntry::Change(*arc)))
                } else {
                    None
                }
            }
            ModelEvent::AddInstance(label, transform) => {
                if let Some(arc) = self.model_store.get(&label) {
                    arc.write().await.add_transform(transform);

                    Some(ChangeListEntry(ModelChangeListEntry::Change(*arc)))
                } else {
                    None
                }
            }
            ModelEvent::RemoveInstance(label, index) => {
                if let Some(arc) = self.model_store.get(&label) {
                    arc.write().await.remove_transform(index);

                    Some(ChangeListEntry(ModelChangeListEntry::Change(*arc)))
                } else {
                    None
                }
            }
        }
    }

    async fn handle_camera_event(&mut self, event: CameraEvent) -> Option<ChangeListEntry> {
        match event {
            CameraEvent::Spawn(camera_descriptor) => {
                let label = camera_descriptor.label.clone();
                let camera = Arc::new(RwLock::new(camera_descriptor));

                self.camera_store.insert(label, camera.clone());

                Some(ChangeListEntry::Camera(CameraChangeListEntry::Spawn(
                    camera,
                )))
            }
            CameraEvent::Despawn(label) => {
                if let Some(camera) = self.camera_store.remove(&label) {
                    Some(ChangeListEntry::Camera(CameraChangeListEntry::Despawn(
                        camera,
                    )))
                } else {
                    None
                }
            }
            CameraEvent::Target(label) => {
                if let Some(camera) = self.camera_store.get(&label) {
                    Some(ChangeListEntry::Camera(CameraChangeListEntry::Target(
                        camera.clone(),
                    )))
                } else {
                    None
                }
            }
            CameraEvent::Transform(label, camera_transform) => {
                if let Some(camera) = self.camera_store.get(&label) {
                    let mut lock = camera.write().await;
                    lock.apply_change(camera_transform);

                    Some(ChangeListEntry::Camera(CameraChangeListEntry::Change(
                        camera.clone(),
                    )))
                } else {
                    None
                }
            }
        }
    }

    async fn handle_clear_event(&mut self) -> Option<ChangeListEntry> {
        self.model_store.clear();
        self.camera_store.clear();

        Some(ChangeListEntry::Clear)
    }
}
