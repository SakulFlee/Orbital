use std::thread::JoinHandle;

use crossbeam_channel::Receiver;

use crate::{error::Error, world::WorldChange};

#[derive(Debug)]
pub struct GLTFWorker {
    pub receiver: Receiver<Result<Vec<WorldChange>, Error>>,
    pub worker: JoinHandle<()>,
}
