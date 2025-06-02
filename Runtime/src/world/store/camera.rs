use std::error::Error;

use hashbrown::HashMap;
use log::warn;
use wgpu::{Device, Queue};

use crate::{
    cache::{Cache, CacheEntry},
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

    pub fn realize_and_cache(
        &mut self,
        ids: Vec<u128>,
        device: &Device,
        queue: &Queue,
    ) -> Vec<(u128, Box<dyn Error>)> {
        let mut errors: Vec<(u128, Box<dyn Error>)> = Vec::new();

        for id in ids {
            if self.cache_realizations.contains_key(&id) {
                continue;
            }

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

    pub fn get_realizations(&mut self, ids: Vec<u128>) -> Vec<&Camera> {
        ids.into_iter()
            .filter_map(|id| match self.cache_realizations.get(&id) {
                Some(model) => Some(model.inner()),
                None => {
                    warn!("Model with id #{id} has not yet been realized! Skipping ...");
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn cleanup(&mut self) {
        self.cache_realizations.cleanup();
    }
}
