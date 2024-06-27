use akimo_runtime::{
    game::{Game, World, WorldChange},
    log::info,
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
        info!("Queuing TestElement spawn");
        world.queue_world_change(WorldChange::SpawnElement(Box::new(TestElement::default())));
    }
}
