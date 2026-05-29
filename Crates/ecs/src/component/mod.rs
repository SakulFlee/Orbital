mod store;
pub use store::*;

mod world_store;
pub use world_store::*;

use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

pub trait Component: Any + Debug {
    fn type_id(&self) -> TypeId {
        Any::type_id(self)
    }
}

impl<T: Any + Debug> Component for T {}
