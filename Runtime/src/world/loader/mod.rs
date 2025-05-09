use std::{error::Error, fmt::Debug};

use super::WorldChange;

mod error;
pub use error::*;

mod executor;
pub use executor::*;

mod gltf;
pub use gltf::*;

// TODO: Replace with `FileManager` + "Binary" Loaders for e.g. glTF
pub trait Loader: Debug + Send + Sync {
    fn begin_processing(&mut self);
    fn is_done_processing(&self) -> bool;
    fn finish_processing(&mut self) -> Result<Vec<WorldChange>, Box<dyn Error + Send + Sync>>;
}
