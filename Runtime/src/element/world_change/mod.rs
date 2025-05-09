use crate::{app::AppChange, element::Message};

mod element;
pub use element::*;

mod model;
pub use model::*;

mod camera;
pub use camera::*;

mod file_manager;
pub use file_manager::*;

#[derive(Debug)]
pub enum WorldChange {
    Model(ModelChange),
    Camera(CameraChange),
    Element(ElementChange),
    App(AppChange),
    FileManager(FileManager),
    SendMessage(Message),
    Clean,
    Clear = Self::Clean,
}
