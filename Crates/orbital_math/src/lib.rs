//! # Math Module
//!
//! Generic mathematical helpers shared across the engine: transforms and
//! wgpu/WebGPU clip-space projection matrices.

mod mode;
pub use mode::*;

mod projection;
pub use projection::*;

mod transform;
pub use transform::*;
