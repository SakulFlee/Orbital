use winit::error::EventLoopError;

use crate::{ConnectorError, EntityError};

#[derive(Debug)]
pub enum RuntimeError {
    EventLoopError(EventLoopError),
    ConnectorError(ConnectorError),
    EntityError(EntityError),
    MutexPoisonError(String),
}
