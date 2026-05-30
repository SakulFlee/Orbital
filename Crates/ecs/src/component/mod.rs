mod store;
pub use store::*;

mod world_store;
pub use world_store::*;

use std::{any::Any, fmt::Debug};

pub trait Component: Any + Debug {
    fn make_store(&self) -> ComponentStore<Self>
    where
        Self: Sized,
    {
        ComponentStore::new()
    }
}

impl<T: Any + Debug> Component for T {}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};

    use crate::{Component, ComponentStore};

    #[derive(Debug)]
    struct TestComponent;

    #[test]
    fn store_creation() {
        let test_component = TestComponent;
        let store = test_component.make_store();
        assert_eq!(
            TypeId::of::<ComponentStore<TestComponent>>(),
            store.type_id()
        );
    }
}
