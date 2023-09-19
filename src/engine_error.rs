#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum EngineError {
    NoAdapters,
    RequestDeviceError,
    CreateSurfaceError,
    NoMatch,
}
