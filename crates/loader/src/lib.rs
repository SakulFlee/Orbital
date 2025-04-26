use std::fmt::Debug;

use world_change::WorldChange;

pub trait Loader: Debug + Send + Sync {
    type Error: Debug + Send + Sync;

    fn begin_processing(&mut self);
    fn is_done_processing(&self) -> bool;
    fn finish_processing(&mut self) -> Result<Vec<WorldChange>, Self::Error>;
}
