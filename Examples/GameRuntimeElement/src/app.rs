use akimo_runtime::{
    game::{Game, World, WorldChange},
    log::debug,
    resources::descriptors::{CompositionDescriptor, ImportDescriptor},
};

use crate::element::TestElement;

pub struct ExampleGame;

impl Game for ExampleGame {
    fn init() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn on_startup(&mut self, world: &mut World)
    where
        Self: Sized,
    {
        debug!("REGISTRATION");
        world.register_element(TestElement::default())
    }

    fn on_update(&mut self, _delta_time: f64, world: &mut World)
    where
        Self: Sized,
    {
        // TODO uhhhhh....
    }
}
