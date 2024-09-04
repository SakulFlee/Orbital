mod gltf;
pub use gltf::*;

use crate::{error::Error, game::WorldChange};

pub trait Worker {
    fn begin_processing(&mut self);
    fn is_done_processing(&self) -> bool;
    fn finish_processing(&mut self) -> Result<Vec<WorldChange>, Error>;
}
