use wgpu::CreateSurfaceError;

#[derive(Debug)]
pub enum ConnectorError {
    NoAdapters,
    RequestDeviceError,
    NoMatch,
    SurfaceError(CreateSurfaceError),
}
