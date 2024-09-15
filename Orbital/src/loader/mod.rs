use std::fmt::Debug;

use crate::{error::Error, game::WorldChange};

mod gltf;
pub use gltf::*;

pub trait Loader: Debug {
    fn begin_processing(&mut self);
    fn is_done_processing(&self) -> bool;
    fn finish_processing(&mut self) -> Result<Vec<WorldChange>, Error>;
}
