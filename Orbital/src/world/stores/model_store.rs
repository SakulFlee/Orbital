use std::sync::Arc;

use async_std::sync::RwLock;
use hashbrown::HashMap;

use crate::resources::descriptors::ModelDescriptor;

#[derive(Debug)]
pub struct ModelStore
where
    Self: Send + Sync,
{
    models: HashMap<String, Arc<RwLock<ModelDescriptor>>>,
}

impl Default for ModelStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelStore {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub async fn add(&mut self, model_descriptor: ModelDescriptor) {
        let label = model_descriptor.label.clone();
        self.models
            .insert(label, Arc::new(RwLock::new(model_descriptor)));
    }

    pub async fn remove(&mut self, label: &str) {
        self.models.remove(label);
    }

    pub fn get(&self, label: &str) -> Option<&Arc<RwLock<ModelDescriptor>>> {
        self.models.get(label)
    }

    pub fn get_all(&self) -> Vec<&Arc<RwLock<ModelDescriptor>>> {
        self.models.values().collect()
    }

    pub fn clear(&mut self) {
        self.models.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}
