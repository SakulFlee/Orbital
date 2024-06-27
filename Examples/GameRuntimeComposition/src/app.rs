use akimo_runtime::{
    game::{Game, World, WorldChange},
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

    fn on_update(&mut self, _delta_time: f64, world: &mut World)
    where
        Self: Sized,
    {
        if world.composition.is_empty() {
            world.queue_world_change(WorldChange::SwitchComposition(
                CompositionDescriptor::FromGLTF(
                    "Assets/Models/Cubes.glb",
                    ImportDescriptor::Index(0),
                ),
            ));
        }
    }
}
