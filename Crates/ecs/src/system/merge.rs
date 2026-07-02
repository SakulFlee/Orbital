use crate::ComponentStore;

pub struct Snapshot<T> {
    pub sparse: Vec<Option<usize>>,
    pub dense: Vec<usize>,
    pub components: Vec<T>,
}

impl<T: Clone> Snapshot<T> {
    pub fn clone_from_store(store: &ComponentStore<T>) -> Self {
        Self {
            sparse: store.sparse.clone(),
            dense: store.dense.clone(),
            components: store.components.clone(),
        }
    }
}

impl<T: std::fmt::Debug + Send + Sync + 'static> Snapshot<T> {
    pub fn merge_into(self, world: &crate::World) {
        let store = world
            .get_component_store_mut::<T>()
            .expect("ComponentStore disappeared during snapshot");
        let typed = store.get_mut_store();
        typed.sparse = self.sparse;
        typed.dense = self.dense;
        typed.components = self.components;
    }
}
