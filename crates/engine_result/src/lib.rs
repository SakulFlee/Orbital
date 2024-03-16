pub use engine_error::EngineError;

pub type EngineResult<ReturnType, SurfaceError> = Result<ReturnType, EngineError<SurfaceError>>;
