pub use engine_result::EngineResult;
use wgpu::SurfaceError;

pub type EngineResultWGPU<ReturnType> = EngineResult<ReturnType, SurfaceError>;
