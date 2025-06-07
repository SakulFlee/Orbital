use std::{cell::RefCell, error::Error, sync::Arc};

use hashbrown::{hash_map::Values, HashMap};
use log::warn;
use wgpu::{Device, Queue, TextureFormat};

use crate::{
    cache::{Cache, CacheEntry},
    element::ModelEvent,
    or::Or,
    resources::{
        BoundingBox, MaterialShader, MaterialShaderDescriptor, Mesh, MeshDescriptor, Model,
        ModelDescriptor,
    },
};

use super::StoreError;

#[derive(Debug, Default)]
pub struct ModelStore {
    map_descriptors: HashMap<u128, ModelDescriptor>,
    cache_realizations: Cache<u128, Model>,
    // Descriptors that are queued to be realized
    queue_realizations: Vec<u128>,
    queue_bounding_boxes: Vec<u128>,
    map_bounding_boxes: HashMap<u128, BoundingBox>, // TODO: WIP
    map_label: HashMap<String, u128>,
    id_counter: u128, // TODO: Is that high of a number needed? u64, u32, ...
    // TODO: Dynamic number type that increases in size automatically?
    free_ids: Vec<u128>,
    cache_mesh: RefCell<Cache<Arc<MeshDescriptor>, Mesh>>,
    cache_material: RefCell<Cache<Arc<MaterialShaderDescriptor>, MaterialShader>>,
}

impl ModelStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store(&mut self, descriptor: ModelDescriptor) {
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
            if self.map_bounding_boxes.remove(&idx).is_none() {
                panic!("ModelStore Desync! No associated BoundingBox found!");
            }

            // Must also exist!
            if self.map_label.remove(&descriptor.label).is_none() {
                panic!("ModelStore Desync! No associated Label found!");
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

    pub fn get_bounding_boxes(&self) -> Values<u128, BoundingBox> {
        self.map_bounding_boxes.values()
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

    pub fn process_bounding_boxes(&mut self, device: &Device) {
        for id in self
            .queue_bounding_boxes
            .drain(0..self.queue_bounding_boxes.len())
        {
            let descriptor = match self.map_descriptors.get(&id) {
                Some(x) => x,
                None => {
                    warn!("Attempting to process BoundingBox for id #{id}, but Descriptor cannot be found!");
                    continue;
                }
            };

            let bounding_box_descriptor = descriptor.mesh.find_bounding_box();
            let bounding_box = BoundingBox::new(&bounding_box_descriptor, device);
            self.map_bounding_boxes.insert(id, bounding_box);
        }
    }

    pub fn realize_and_cache(
        &mut self,
        surface_format: &TextureFormat,
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

            let model = match Model::from_descriptor(
                descriptor,
                surface_format,
                device,
                queue,
                &self.cache_mesh,
                &self.cache_material,
            ) {
                Ok(model) => model,
                Err(e) => {
                    errors.push((id, Box::new(e)));
                    continue;
                }
            };

            let cache_entry = CacheEntry::new(model);
            self.cache_realizations.insert(id, cache_entry);
        }

        errors
    }

    pub fn get_realizations(&mut self, ids: Vec<u128>) -> Vec<&Model> {
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
        self.cache_mesh.borrow_mut().cleanup();
        self.cache_material.borrow_mut().cleanup();
    }

    pub fn clear(&mut self) {
        self.map_label.clear();
        self.map_descriptors.clear();
        self.map_bounding_boxes.clear();
        self.cache_realizations.clear();
        self.cache_mesh.borrow_mut().clear();
        self.cache_material.borrow_mut().clear();
        self.free_ids.clear();
        self.id_counter = 0;
    }
    }

    pub fn handle_event(&mut self, model_event: ModelEvent) {
        match model_event {
            ModelEvent::Spawn(descriptor) => {
                self.store(descriptor);
            }
            ModelEvent::Despawn(label) => {
                self.remove(Or::Left(&label));
            }
            ModelEvent::Transform(label, mode) => {
                if let Some(idx) = self.label_to_id(&label) {
                    let descriptor = self.map_descriptors.get_mut(&idx).unwrap();
                    descriptor.apply_transform(mode);

                    if self.cache_realizations.contains_key(&idx) {
                        self.flag_realization(vec![idx], true);
                    }
                } else {
                    warn!(
                        "Attempting to modify Model with label '{label}', which cannot be found!"
                    );
                }
            }
            ModelEvent::TransformInstance(label, mode, transform_idx) => {
                if let Some(idx) = self.label_to_id(&label) {
                    let descriptor = self.map_descriptors.get_mut(&idx).unwrap();
                    descriptor.apply_transform_specific(mode, transform_idx);

                    if self.cache_realizations.contains_key(&idx) {
                        self.flag_realization(vec![idx], true);
                    }
                } else {
                    warn!(
                        "Attempting to modify Model with label '{label}', which cannot be found!"
                    );
                }
            }
            ModelEvent::AddInstance(label, transform) => {
                if let Some(idx) = self.label_to_id(&label) {
                    let descriptor = self.map_descriptors.get_mut(&idx).unwrap();
                    descriptor.add_transform(transform);

                    if self.cache_realizations.contains_key(&idx) {
                        self.flag_realization(vec![idx], true);
                    }
                } else {
                    warn!(
                        "Attempting to add instance to Model with label '{label}', which cannot be found!"
                    );
                }
            }
            ModelEvent::RemoveInstance(label, transform_idx) => {
                if let Some(idx) = self.label_to_id(&label) {
                    let descriptor = self.map_descriptors.get_mut(&idx).unwrap();
                    descriptor.remove_transform(transform_idx);

                    if self.cache_realizations.contains_key(&idx) {
                        self.flag_realization(vec![idx], true);
                    }
                } else {
                    warn!(
                        "Attempting to add instance to Model with label '{label}', which cannot be found!"
                    );
                }
            }
        }
    }
}
