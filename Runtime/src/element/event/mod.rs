use crate::app::RuntimeEvent;

mod element;
pub use element::*;

mod world;
pub use world::*;

mod file_manager;
pub use file_manager::*;

#[derive(Debug)]
pub enum Event {
    Element(ElementEvent),
    World(WorldEvent),
    App(RuntimeEvent), // TODO
}
