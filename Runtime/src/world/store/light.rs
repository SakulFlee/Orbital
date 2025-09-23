use std::error::Error;

use hashbrown::HashMap;
use log::warn;
use ulid::Ulid;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferDescriptor, BufferUsages, Device, Queue,
};

use crate::{
    cache::{Cache, CacheEntry},
    element::LightEvent,
    or::Or,
    resources::{Light, LightDescriptor},
};

use super::StoreError;

#[derive(Debug, Default)]
pub struct LightStore {
    map_label: HashMap<String, Ulid>,
    map_descriptors: HashMap<Ulid, LightDescriptor>,
    cache_realizations: Cache<Ulid, Light>,
    queue_realizations: Vec<Ulid>,
    light_buffer: Option<Buffer>,
}

impl LightStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, descriptor: LightDescriptor) {
        let id = Ulid::new();

        self.map_label.insert(descriptor.label.clone(), id);
        self.map_descriptors.insert(id, descriptor);
    }

    pub fn remove(&mut self, id: Or<&str, Ulid>) -> bool {
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
                panic!("LightStore Desync! No associated Label found!");
            }

            true
        } else {
            false
        }
    }

    pub fn label_to_id(&self, label: &str) -> Option<Ulid> {
        self.map_label.get(label).copied()
    }

    pub fn id_to_label(&self, id: Ulid) -> Option<&str> {
        self.map_descriptors
            .get(&id)
            .map(|descriptor| descriptor.label.as_str())
    }

    pub fn flag_realization(&mut self, ids: Vec<Ulid>, update_existing: bool) {
        for id in ids {
            if self.cache_realizations.contains_key(&id) && !update_existing {
                // Skip any existing realizations if we aren't updating existing entries.
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

    pub fn realize_and_cache(&mut self, device: &Device, queue: &Queue) -> Vec<Box<dyn Error>> {
        let mut errors: Vec<Box<dyn Error>> = Vec::new();

        for id in self
            .queue_realizations
            .drain(0..self.queue_realizations.len())
        {
            let descriptor = match self.map_descriptors.get(&id) {
                Some(descriptor) => descriptor,
                None => {
                    errors.push(Box::new(StoreError::InvalidIndex { index: id }));
                    continue;
                }
            };

            // Recreate the Light and replace it inside our cache
            match Light::from_descriptor(descriptor, device, queue) {
                Ok(light) => {
                    let cache_entry = CacheEntry::new(light);
                    self.cache_realizations.insert(id, cache_entry);
                }
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            }
        }

        errors
    }

    pub fn get_realizations(&self, ids: Vec<Ulid>) -> Vec<&Light> {
        ids.into_iter()
            .filter_map(|id| match self.cache_realizations.get(&id) {
                Some(light) => Some(light.inner()),
                None => {
                    warn!("Light with id #{id} has not yet been realized! Skipping ...");
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn cleanup(&mut self) {
        self.cache_realizations.cleanup();
    }

    pub fn clear(&mut self) {
        self.map_descriptors.clear();
        self.map_label.clear();
        self.cache_realizations.clear();
    }

    pub fn create_light_buffer(&mut self, device: &Device, queue: &Queue) {
        // Create a buffer with all light data
        let light_count = self.map_descriptors.len();
        if light_count == 0 {
            // Create an empty buffer with sufficient size to satisfy shader requirements
            // Shader expects at least 64 bytes for the light buffer
            self.light_buffer = Some(device.create_buffer(&BufferDescriptor {
                label: Some("Light Storage Buffer"),
                size: 64, // Minimum size that satisfies shader requirements
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            return;
        }

        // Calculate buffer size: Each light needs 64 bytes (position: 16, color: 16, direction: 16, params: 16)
        let light_size = 64;
        let buffer_size = (light_count * light_size) as u64;

        // Create buffer data
        let mut buffer_data = Vec::new();
        for descriptor in self.map_descriptors.values() {
            buffer_data.extend_from_slice(&descriptor.to_buffer_data());
        }

        // Pad buffer data to multiple of 4 bytes if needed
        while buffer_data.len() % 4 != 0 {
            buffer_data.push(0);
        }

        let light_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Light Storage Buffer"),
            contents: &buffer_data,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        self.light_buffer = Some(light_buffer);
    }

    pub fn light_buffer(&self) -> Option<&Buffer> {
        self.light_buffer.as_ref()
    }

    pub fn handle_event(&mut self, light_event: LightEvent) {
        match light_event {
            LightEvent::Spawn(light_descriptor) => {
                self.store(light_descriptor);
            }
            LightEvent::Despawn(label) => {
                self.remove(Or::Left(&label));
            }
            LightEvent::Update(label, new_descriptor) => {
                let id = match self.label_to_id(&label) {
                    Some(x) => x,
                    None => {
                        warn!("Attempting to update Light with label '{label}', but label cannot be found!");
                        return;
                    }
                };

                self.map_descriptors.insert(id, new_descriptor);

                if self.cache_realizations.contains_key(&id) {
                    self.flag_realization(vec![id], true);
                }
            }
        }
    }
}
