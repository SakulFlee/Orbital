use crate::{app::RuntimeEvent, element::Message};

mod element;
pub use element::*;

mod model;
pub use model::*;

mod camera;
pub use camera::*;

mod file_manager;
pub use file_manager::*;

#[derive(Debug)]
pub enum Event {
    Model(ModelEvent),
    Camera(CameraEvent),
    Element(ElementEvent),
    App(RuntimeEvent),
    File(FileEvent),
    Clear,
}
