use std::{error::Error, thread::JoinHandle};

use crossbeam_channel::Receiver;

use super::Event;

#[derive(Debug)]
pub struct GLTFWorker {
    pub receiver: Receiver<Result<Vec<Event>, Box<dyn Error + Send + Sync>>>,
    pub worker: JoinHandle<()>,
}
