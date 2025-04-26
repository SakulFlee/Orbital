use std::{error::Error, fmt::Debug};

use world_change::WorldChange;

mod error;
pub use error::*;

pub trait Loader: Debug + Send + Sync {
    fn begin_processing(&mut self);
    fn is_done_processing(&self) -> bool;
    fn finish_processing(&mut self) -> Result<Vec<WorldChange>, Box<dyn Error + Send + Sync>>;
}
