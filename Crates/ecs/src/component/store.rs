#[derive(Debug)]
pub struct ComponentStore<T> {
    /// [Entity ID] -> [Component Index]
    /// Maps Entity ID to Component index in a sparse set.
    /// Used for accessing a Component of a given Entity identified by an ID.
    pub sparse: Vec<Option<usize>>,
    /// [Component Index] -> [Entity ID]
    /// Maps Component index back to Entity IDs in a dense set.
    /// Used for detaching/removing components to have a "reverse lookup" available.
    pub dense: Vec<usize>,
    /// [Component Index] -> [Component]
    /// Stores the actual Components in a dense set.
    pub components: Vec<T>,
}

impl<T> ComponentStore<T> {
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn attach(&mut self, entity_id: usize, component: T) {
        // Aquire next index
        let next_index = self.components.len();

        // Push to dense arrays
        self.components.push(component);
        self.dense.push(entity_id);

        // Update sparse map
        if entity_id >= self.sparse.len() {
            self.sparse.resize(entity_id + 1, None);
        }
        self.sparse[entity_id] = Some(next_index);
    }

    pub fn detach(&mut self, entity_id: usize) -> Option<T> {
        if entity_id >= self.sparse.len() {
            // Cannot logically be present
            return None;
        }

        // Retrieve component access, return None if it doesn't exist
        let component_index = self.sparse[entity_id]?;

        // Clear sparse index
        self.sparse[entity_id] = None;

        // Swap remove on both dense arrays
        let component = self.components.swap_remove(component_index);
        self.dense.swap_remove(component_index);

        if component_index < self.components.len() {
            let moved_entity = self.dense[component_index];

            self.sparse[moved_entity] = Some(component_index);
        }

        Some(component)
    }

    pub fn get_component(&self, entity_id: usize) -> Option<&T> {
        if entity_id >= self.sparse.len() {
            // Cannot logically be present
            return None;
        }

        let component_index = self.sparse[entity_id]?;
        self.components.get(component_index)
    }
}

#[cfg(test)]
mod tests {
    use crate::ComponentStore;

    #[test]
    fn component_values_correct() {
        let mut store = ComponentStore::<usize>::new();

        store.attach(0, 123);
        store.attach(100, 321);
        store.attach(500, 333);

        assert_eq!(store.get_component(0), Some(&123));
        assert_eq!(store.get_component(100), Some(&321));
        assert_eq!(store.get_component(500), Some(&333));
    }

    #[test]
    fn component_invalid_entity() {
        let mut store = ComponentStore::<usize>::new();

        store.attach(0, 123);
        store.attach(100, 321);
        store.attach(500, 333);

        assert_eq!(store.get_component(1), None);
        assert_eq!(store.get_component(99), None);
        assert_eq!(store.get_component(501), None);
    }

    #[test]
    fn component_gone_after_removing() {
        let mut store = ComponentStore::<usize>::new();

        store.attach(0, 123);
        store.attach(100, 321);
        store.attach(500, 333);

        store.detach(100);

        assert_eq!(store.get_component(100), None);
    }
}
