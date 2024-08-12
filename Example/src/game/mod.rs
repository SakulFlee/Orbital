use elements::{
    camera::Camera, damaged_helmet::DamagedHelmet, pbr_spheres::PBRSpheres,
    ping_pong::PingPongElement,
};
use orbital::{
    game::{Game, World, WorldChange},
    log::info,
};

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
        info!("Queuing Camera spawn");
        world.process_world_change(WorldChange::SpawnElement(Box::new(Camera::new())));

        info!("Queuing Ping & Pong spawn");
        world.process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
            true,
        ))));
        world.process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
            false,
        ))));

        world.process_world_change(WorldChange::SpawnElement(Box::new(DamagedHelmet {})));
    }
}
