mod model;
pub use model::*;

mod camera;
pub use camera::*;

#[derive(Debug)]
pub enum ChangeListEntry {
    Model(ModelChangeListEntry),
    Camera(CameraChangeListEntry),
    Clear,
}
