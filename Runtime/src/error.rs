use wgpu::CreateSurfaceError;
use winit::error::EventLoopError;

#[derive(Debug)]
pub enum ConnectorError {
    NoAdapters,
    RequestDeviceError,
    NoMatch,
    SurfaceError(CreateSurfaceError),
}

#[derive(Debug)]
pub enum RuntimeError {
    EventLoopError(EventLoopError),
    ConnectorError(ConnectorError),
    EntityError(EntityError),
    MutexPoisonError(String),
}

#[derive(Debug)]
pub enum EntityError {
    EntityExistsAlready,
}
