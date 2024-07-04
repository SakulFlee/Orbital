use akimo_runtime::game::{Game, World, WorldChange};

pub mod cube;
use cube::Cube;

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
        // We first need to make an Element.
        // This is because a Model is associated with a Element ID (ULID).
        // A Game itself doesn't have such an ID as of now.
        // Note: Maybe it does in the future! Check the docs and open a PR if
        // it does and I forgot to update this! <3
        //
        // Point being, we need an Element to spawn our Cubes in, so that they
        // get associated with an Element.
        // Check the Cube::on_registration function for more!
        let cube_element = Cube {};

        // Since Elements are dynamic traits, we have to box it
        // (i.e. place on heap for dynamic access)
        let boxed_cube = Box::new(cube_element);

        // Finally, we can make the WorldChange and queue the Element spawning.
        let world_change = WorldChange::SpawnElement(boxed_cube);
        world.queue_world_change(world_change);
    }
}
