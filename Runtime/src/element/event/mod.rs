use crate::app::AppEvent;

mod element;
pub use element::*;

mod world;
pub use world::*;

#[derive(Debug)]
pub enum Event {
    Element(ElementEvent),
    World(WorldEvent),
    App(AppEvent),
}
