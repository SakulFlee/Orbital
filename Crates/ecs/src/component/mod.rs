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
