use std::sync::Arc;

use async_std::sync::RwLock;
use hashbrown::HashMap;

mod model;
pub use model::*;

mod camera;
pub use camera::*;

pub type Store<T> = HashMap<String, Arc<RwLock<T>>>;
