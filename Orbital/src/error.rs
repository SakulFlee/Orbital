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
    GltfError(Box<dyn std::error::Error + Send + Sync>),
    NoIndices,
    SceneNotFound,
    ModelNotFound,
    ImageError(image::ImageError),
    CannotRealizeTag(String),
    WrongFormat,
    BindGroupMissing,
    FileNotFound,
    NotDoneProcessing,
    CrossbeamRecvError(crossbeam_channel::RecvError),
    NotFound,
    TomlError(toml::de::Error),
    SerdeJsonError(serde_json::Error),
}
