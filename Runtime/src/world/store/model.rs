use std::{cell::RefCell, error::Error, sync::Arc};

use hashbrown::{hash_map::Values, HashMap};
use log::warn;
use wgpu::{Device, Queue, TextureFormat};

use crate::{
    cache::{Cache, CacheEntry},
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
    map_bounding_boxes: HashMap<u128, BoundingBox>,
    map_label: HashMap<String, u128>,
    id_counter: u128,
    free_ids: Vec<u128>,
    cache_mesh: RefCell<Cache<Arc<MeshDescriptor>, Mesh>>,
    cache_material: RefCell<Cache<Arc<MaterialShaderDescriptor>, MaterialShader>>,
}

impl ModelStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store_model(&mut self, descriptor: ModelDescriptor, device: &Device) {
        let id = match self.free_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.id_counter;
                self.id_counter += 1;
                id
            }
        };

        let bounding_box_descriptor = descriptor.mesh.find_bounding_box();
        let bounding_box = BoundingBox::new(&bounding_box_descriptor, device);

        self.map_label.insert(descriptor.label.clone(), id);
        self.map_descriptors.insert(id, descriptor);
        self.map_bounding_boxes.insert(id, bounding_box);
    }

    pub fn remove_model(&mut self, id: Or<&str, u128>) -> bool {
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

            // Mus also exist!
            if self.map_label.remove(&descriptor.label).is_none() {
                panic!("ModelStore Desync! No associated Label found!");
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
            .get(&id).map(|descriptor| descriptor.label.as_str())
    }

    pub fn get_bounding_boxes(&self) -> Values<u128, BoundingBox> {
        self.map_bounding_boxes.values()
    }

    pub fn realize_and_cache_models(
        &mut self,
        ids: Vec<u128>,
        surface_format: &TextureFormat,
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

    pub fn get_realized_models(&mut self, ids: Vec<u128>) -> Vec<&Model> {
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
}
