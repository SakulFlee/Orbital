use crate::EntityIdType;

#[derive(Debug)]
pub struct ComponentStore<T> {
    pub entity_map: Vec<EntityIdType>,
    pub components: Vec<T>,
}

impl<T> ComponentStore<T> {
    pub fn new() -> Self {
        Self {
            entity_map: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn attach_component(&mut self, entity_id: EntityIdType, component: T) {
        let next_index = self.components.len();
        self.components.push(component);

        if entity_id >= self.entity_map.len() {
            let current_len = self.entity_map.len();
            let new_len = entity_id + 1 - current_len;
            self.entity_map.resize(new_len, 0);
        }
        self.entity_map[entity_id] = next_index;
    }

    pub fn detach_component(&mut self, entity_id: EntityIdType) -> Option<T> {
        if entity_id >= self.entity_map.len() {
            // Cannot logically be present
            return None;
        }

        let component_index = self.entity_map[entity_id];
        if component_index > 0 {
            self.entity_map[entity_id] = 0;
            let component = self.components.swap_remove(component_index);
            return Some(component);
        }
        None
    }

    pub fn get_component(&self, entity_id: EntityIdType) -> Option<&T> {
        if entity_id >= self.entity_map.len() {
            // Cannot logically be present
            return None;
        }

        let component_index = self.entity_map[entity_id];
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
