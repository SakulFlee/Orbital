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
pub enum WindowError {
    EventLoopError(EventLoopError),
}

#[derive(Debug)]
pub enum RuntimeError {
    WindowError(WindowError),
    ConnectorError(ConnectorError),
}
