use akimo_runtime::{
    game::{Game, World},
    log::debug,
};

pub struct ExampleGame {}

impl Game for ExampleGame {
    fn init() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn on_update(&mut self, delta_time: f64, _world: &mut World)
    where
        Self: Sized,
    {
        debug!("Update :: {} ms", delta_time);
    }
}
