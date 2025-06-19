use std::error::Error;

use hashbrown::HashMap;
use log::{error, warn};
use wgpu::{Device, Queue};

use crate::{
    cache::{Cache, CacheEntry},
    element::CameraEvent,
    or::Or,
    resources::{Camera, CameraDescriptor},
};

use super::StoreError;

#[derive(Debug, Default)]
pub struct CameraStore {
    id_counter: u128,
    free_ids: Vec<u128>,
    map_label: HashMap<String, u128>,
    map_descriptors: HashMap<u128, CameraDescriptor>,
    cache_realizations: Cache<u128, Camera>,
    queue_realizations: Vec<u128>,
    active_camera: u128,
}

impl CameraStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, descriptor: CameraDescriptor) {
        let id = match self.free_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.id_counter;
                self.id_counter += 1;
                id
            }
        };

        self.map_label.insert(descriptor.label.clone(), id);
        self.map_descriptors.insert(id, descriptor);
    }

    pub fn remove(&mut self, id: Or<&str, u128>) -> bool {
        let idx = match id {
            Or::Left(label) => match self.label_to_id(label) {
                Some(id) => id,
                None => return false,
            },
            Or::Right(id) => id,
        };

        if let Some(descriptor) = self.map_descriptors.remove(&idx) {
            // Possibly, might not exist!
            self.cache_realizations.remove(&idx);

            // Must exist!
            if self.map_label.remove(&descriptor.label).is_none() {
                panic!("CameraStore Desync! No associated Label found!");
            }

            if idx <= self.id_counter {
                self.free_ids.push(idx);
            }

            true
        } else {
            false
        }
    }

    pub fn label_to_id(&self, label: &str) -> Option<u128> {
        self.map_label.get(label).copied()
    }

    pub fn id_to_label(&self, id: u128) -> Option<&str> {
        self.map_descriptors
            .get(&id)
            .map(|descriptor| descriptor.label.as_str())
    }

    pub fn target_camera(&mut self, id: u128) {
        if !self.map_descriptors.contains_key(&id) {
            error!("Attempting to target a Camera with id #{id}, which doesn't exist!");
            return;
        }

        self.active_camera = id;
        self.flag_realization(vec![id], true);
    }

    pub fn flag_realization(&mut self, ids: Vec<u128>, update_existing: bool) {
        for id in ids {
            if self.cache_realizations.contains_key(&id) && !update_existing {
                // Skip any existing realisations if we aren't updating existing entries.
                continue;
            }

            // Filter out any non-existing descriptors
            if !self.map_descriptors.contains_key(&id) {
                warn!("Attempting to flag realization for non existing descriptor with id #{id}!");
                continue;
            }

            self.queue_realizations.push(id);
        }
    }

    pub fn realize_and_cache(
        &mut self,
        device: &Device,
        queue: &Queue,
    ) -> Vec<(u128, Box<dyn Error>)> {
        let mut errors: Vec<(u128, Box<dyn Error>)> = Vec::new();

        for id in self
            .queue_realizations
            .drain(0..self.queue_realizations.len())
        {
            let descriptor = match self.map_descriptors.get(&id) {
                Some(descriptor) => descriptor,
                None => {
                    errors.push((id, Box::new(StoreError::InvalidIndex { index: id })));
                    continue;
                }
            };

            let camera = Camera::from_descriptor(descriptor.clone(), device, queue);

            let cache_entry = CacheEntry::new(camera);
            self.cache_realizations.insert(id, cache_entry);
        }

        errors
    }

    pub fn realize_and_cache_active_camera(
        &mut self,
        device: &Device,
        queue: &Queue,
    ) -> Option<(u128, Box<dyn Error>)> {
        // There can only be one result, if at all
        self.flag_realization(vec![self.active_camera], true);
        self.realize_and_cache(device, queue).pop()
    }

    pub fn get_realizations(&self, ids: Vec<u128>) -> Vec<&Camera> {
        ids.into_iter()
            .filter_map(|id| match self.cache_realizations.get(&id) {
                Some(model) => Some(model.inner()),
                None => {
                    warn!("Camera with id #{id} has not yet been realized! Skipping ...");
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn get_realized_active_camera(&self) -> Option<&Camera> {
        self.get_realizations(vec![self.active_camera]).pop()
    }

    pub fn cleanup(&mut self) {
        self.cache_realizations.cleanup();
    }

    pub fn clear(&mut self) {
        self.map_descriptors.clear();
        self.map_label.clear();
        self.cache_realizations.clear();
        self.free_ids.clear();
        self.id_counter = 0;
    }

    pub fn handle_event(&mut self, camera_event: CameraEvent) {
        match camera_event {
            CameraEvent::Spawn(camera_descriptor) => {
                self.store(camera_descriptor);
            }
            CameraEvent::Despawn(label) => {
                self.remove(Or::Left(&label));
            }
            CameraEvent::Target(label) => {
                match self.label_to_id(&label) {
                    Some(id) => self.target_camera(id),
                    None => warn!("Attempting to target Camera with label '{label}', but Descriptor does not exist!"),
                }
            },
            CameraEvent::Transform(label, camera_transform) => {
                let id = match self.label_to_id(&label) {
                    Some(x) => x,
                    None => {
                        warn!("Attempting to transform Camera with label '{label}', but label cannot be found!");
                        return;
                    },
                };

                let descriptor = match self.map_descriptors.get_mut(&id) {
                    Some(x) => x,
                    None => {
                        warn!("Attempting to transform Camera with label '{label}', but Descriptor does not exist!");
                        return;
                    },
                };

                descriptor.apply_change(camera_transform);

                if self.cache_realizations.contains_key(&id) {
                    self.flag_realization(vec![id], true);
                }
            },
        }
    }
}
