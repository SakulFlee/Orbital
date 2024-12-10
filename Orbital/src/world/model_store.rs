use hashbrown::HashMap;

use crate::resources::descriptors::ModelDescriptor;

#[derive(Debug)]
pub struct ModelStore {
    models: HashMap<String, ModelDescriptor>,
}

impl ModelStore {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub async fn add(&mut self, model_descriptor: ModelDescriptor) {
        let label = model_descriptor.label.clone();
        self.models.insert(label, model_descriptor);
    }

    pub async fn remove(&mut self, label: &str) {
        self.models.remove(label);
    }

    pub fn get(&self, label: &str) -> Option<&ModelDescriptor> {
        self.models.get(label)
    }

    pub fn get_mut(&mut self, label: &str) -> Option<&mut ModelDescriptor> {
        self.models.get_mut(label)
    }

    pub fn get_all(&self) -> Vec<&ModelDescriptor> {
        self.models.values().collect()
    }

    pub fn clear(&mut self) {
        self.models.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}
