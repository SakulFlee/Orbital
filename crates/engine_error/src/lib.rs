#[derive(Debug)]
pub enum EngineError<SurfaceError> {
    NoAdapters,
    RequestDeviceError,
    CreateSurfaceError,
    NoMatch,
    SurfaceError(SurfaceError),
}
