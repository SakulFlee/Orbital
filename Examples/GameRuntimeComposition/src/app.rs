use akimo_runtime::{
    game::{Game, World, WorldChangeDescriptor},
    resources::descriptors::{CompositionDescriptor, ImportDescriptor},
};

pub struct ExampleGame;

impl Game for ExampleGame {
    fn init() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn cycle(&mut self, _delta_time: f64, world: &mut World)
    where
        Self: Sized,
    {
        if world.composition.is_empty() {
            world.queue_world_change(WorldChangeDescriptor::SwitchComposition(
                CompositionDescriptor::FromGLTF(
                    "Assets/Models/Cubes.glb",
                    ImportDescriptor::Index(0),
                ),
            ));
        }
    }
}
