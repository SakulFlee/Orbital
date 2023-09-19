use wgpu::SurfaceError;

#[derive(Debug, Clone)]
pub enum EngineError {
    NoAdapters,
    RequestDeviceError,
    CreateSurfaceError,
    NoMatch,
    SurfaceError(SurfaceError),
}
