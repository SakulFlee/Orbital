use elements::{
    camera::Camera, damaged_helmet::DamagedHelmet, lights::Lights, pbr_spheres::PBRSpheres,
    ping_pong::PingPongElement,
};
use orbital::game::{Game, World, WorldChange};

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
        // Camera & Lights
        world.process_world_change(WorldChange::SpawnElement(Box::new(Camera::new())));
        world.process_world_change(WorldChange::SpawnElement(Box::new(Lights {})));

        // Ping Pong
        world.process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
            true,
        ))));
        world.process_world_change(WorldChange::SpawnElement(Box::new(PingPongElement::new(
            false,
        ))));

        // Models
        world.process_world_change(WorldChange::SpawnElement(Box::new(PBRSpheres {})));
        world.process_world_change(WorldChange::SpawnElement(Box::new(DamagedHelmet {})));
    }
}
