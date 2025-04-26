use std::{error::Error, thread::JoinHandle};

use crossbeam_channel::Receiver;
use world_change::WorldChange;

#[derive(Debug)]
pub struct GLTFWorker {
    pub receiver: Receiver<Result<Vec<WorldChange>, Box<dyn Error + Send + Sync>>>,
    pub worker: JoinHandle<()>,
}
