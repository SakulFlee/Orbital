use std::{
    error::Error,
    sync::{Arc, RwLock},
};

use hashbrown::HashMap;
use log::warn;
use ulid::Ulid;
use wgpu::{Device, Queue, TextureFormat};

#[cfg(test)]
mod tests;

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
    cache_mesh: RwLock<Cache<Arc<MeshDescriptor>, Mesh>>,
    cache_material: RwLock<Cache<Arc<MaterialShaderDescriptor>, MaterialShader>>,
    // Instancing support
    instance_map: HashMap<u64, u128>, // hash -> base_model_id
    instance_tracker: HashMap<String, (String, Ulid)>, // instance_label -> (base_label, transform_ulid)
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
        self.queue_bounding_boxes.push(id);
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

            // Remove bounding box if it exists (may not be processed yet)
            self.map_bounding_boxes.remove(&idx);

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

    pub fn get_bounding_boxes(&self) -> &HashMap<u128, BoundingBox> {
        &self.map_bounding_boxes
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
    ) -> Vec<(u128, Box<dyn Error + '_>)> {
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
                    errors.push((id, e));
                    continue;
                }
            };

            let cache_entry = CacheEntry::new(model);
            self.cache_realizations.insert(id, cache_entry);
        }

        errors
    }

    pub fn get_realizations(&self, ids: Vec<u128>) -> Vec<&Model> {
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

    pub fn cleanup(&mut self) -> Result<(), Box<dyn Error + '_>> {
        self.cache_realizations.cleanup();
        match self.cache_mesh.write() {
            Ok(mut lock) => lock.cleanup(),
            Err(e) => {
                return Err(Box::new(e));
            }
        }
        match self.cache_material.write() {
            Ok(mut lock) => lock.cleanup(),
            Err(e) => {
                return Err(Box::new(e));
            }
        }

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Box<dyn Error + '_>> {
        match self.cache_mesh.write() {
            Ok(mut lock) => lock.clear(),
            Err(e) => return Err(Box::new(e)),
        };

        match self.cache_material.write() {
            Ok(mut lock) => lock.clear(),
            Err(e) => return Err(Box::new(e)),
        };

        self.map_label.clear();
        self.map_descriptors.clear();
        self.map_bounding_boxes.clear();
        self.cache_realizations.clear();
        self.free_ids.clear();
        self.id_counter = 0;
        self.instance_map.clear();
        self.instance_tracker.clear();

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.map_descriptors.is_empty()
    }

    pub fn handle_event(&mut self, model_event: ModelEvent) {
        match model_event {
            ModelEvent::Spawn(descriptor) => {
                // Check for duplicate models
                let hash = descriptor.instance_hash();
                if let Some(&base_id) = self.instance_map.get(&hash) {
                    // Found duplicate - create instance
                    let base_descriptor = self.map_descriptors.get_mut(&base_id).unwrap();
                    let transform_ulid = base_descriptor
                        .add_transform(*descriptor.transforms.values().next().unwrap());

                    // Use the original label from descriptor, or generate new if it conflicts
                    let instance_label = if self.map_label.contains_key(&descriptor.label) {
                        format!("instance_{}", Ulid::new().to_string())
                    } else {
                        descriptor.label.clone()
                    };
                    let base_label = base_descriptor.label.clone();

                    // Track the instance
                    self.instance_tracker
                        .insert(instance_label.clone(), (base_label, transform_ulid));
                    self.map_label.insert(instance_label, base_id);

                    // Flag for re-realization
                    self.flag_realization(vec![base_id], true);
                } else {
                    // No duplicate - store as base model
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
                    self.instance_map.insert(hash, id);
                    self.queue_bounding_boxes.push(id);
                }
            }
            ModelEvent::Despawn(label) => {
                if let Some((base_label, transform_ulid)) =
                    self.instance_tracker.get(&label).cloned()
                {
                    // This is an instance - remove the specific transform
                    if let Some(base_id) = self.label_to_id(&base_label) {
                        let base_descriptor = self.map_descriptors.get_mut(&base_id).unwrap();
                        base_descriptor.remove_transform(&transform_ulid);
                        self.instance_tracker.remove(&label);
                        self.map_label.remove(&label);

                        // Flag for re-realization
                        self.flag_realization(vec![base_id], true);
                    }
                } else {
                    // This is a base model - remove it and all its instances
                    if let Some(id) = self.label_to_id(&label) {
                        // Remove all instances of this base model
                        let instances_to_remove: Vec<String> = self
                            .instance_tracker
                            .iter()
                            .filter_map(|(inst_label, (base_l, _))| {
                                if base_l == &label {
                                    Some(inst_label.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for inst_label in instances_to_remove {
                            self.instance_tracker.remove(&inst_label);
                            self.map_label.remove(&inst_label);
                        }

                        // Remove the base model
                        self.remove(Or::Right(id));
                    }
                }
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
            ModelEvent::TransformInstance(label, mode, transform_ulid_str) => {
                if let Ok(transform_ulid) = Ulid::from_string(&transform_ulid_str) {
                    if let Some(idx) = self.label_to_id(&label) {
                        let descriptor = self.map_descriptors.get_mut(&idx).unwrap();
                        descriptor.apply_transform_specific(mode, &transform_ulid);

                        if self.cache_realizations.contains_key(&idx) {
                            self.flag_realization(vec![idx], true);
                        }
                    } else {
                        warn!(
                            "Attempting to modify Model with label '{label}', which cannot be found!"
                        );
                    }
                } else {
                    warn!("Invalid ULID string: {}", transform_ulid_str);
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
            ModelEvent::RemoveInstance(label, transform_ulid_str) => {
                if let Ok(transform_ulid) = Ulid::from_string(&transform_ulid_str) {
                    if let Some(idx) = self.label_to_id(&label) {
                        let descriptor = self.map_descriptors.get_mut(&idx).unwrap();
                        descriptor.remove_transform(&transform_ulid);

                        if self.cache_realizations.contains_key(&idx) {
                            self.flag_realization(vec![idx], true);
                        }
                    } else {
                        warn!(
                            "Attempting to remove instance from Model with label '{label}', which cannot be found!"
                        );
                    }
                } else {
                    warn!("Invalid ULID string: {}", transform_ulid_str);
                }
            }
        }
    }
}
