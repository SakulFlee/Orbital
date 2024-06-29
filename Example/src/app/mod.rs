use akimo_runtime::{
    game::{Game, World, WorldChange},
    log::info,
};
use elements::messaging::{ping_pong::PingPongElement, test::TestElement};

pub mod elements;

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

        info!("Queuing Ping & Pong spawn");
        world.queue_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
            true,
        ))));
        world.queue_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
            false,
        ))));
    }
}
