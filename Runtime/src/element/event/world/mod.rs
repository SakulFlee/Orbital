mod model;
pub use model::*;

mod camera;
pub use camera::*;

mod environment;
pub use environment::*;

#[derive(Debug)]
pub enum WorldEvent {
    Model(ModelEvent),
    Camera(CameraEvent),
    Environment(EnvironmentEvent),
    Clear,
}
