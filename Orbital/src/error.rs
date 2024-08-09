use wgpu::CreateSurfaceError;
use winit::error::EventLoopError;

#[derive(Debug)]
pub enum Error {
    EntityExistsAlready,
    NoAdapters,
    RequestDeviceError,
    NoMatch,
    SurfaceError(CreateSurfaceError),
    EventLoopError(EventLoopError),
    MutexPoisonError(String),
    IOError(std::io::Error),
    GltfError(Box<dyn std::error::Error>),
    NoIndices,
    SceneNotFound,
    ModelNotFound,
    ImageError(image::ImageError),
    CannotRealizeTag(String),
    WrongFormat,
}
