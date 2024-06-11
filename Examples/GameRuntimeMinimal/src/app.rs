use akimo_runtime::{game::Game, log::debug, server::RenderServer};

pub struct ExampleGame {}

impl Game for ExampleGame {
    fn init() -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn cycle(&mut self, delta_time: f64, render_server: &mut RenderServer)
    where
        Self: Sized,
    {
        // debug!("Update :: {} ms", delta_time);
    }
}
