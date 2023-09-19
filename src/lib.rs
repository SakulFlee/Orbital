pub mod app;
pub use app::*;

pub mod _engine;
pub use _engine::*;

pub mod engine;

pub mod log;
pub use log::*;

pub mod camera;
pub use camera::*;

pub const APP_NAME: &'static str = "WGPU-Engine";
