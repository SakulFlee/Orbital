pub mod runtime;
pub use runtime::*;

pub mod settings;
pub use settings::*;

pub mod world;
pub use world::*;

pub trait Game {
    fn init() -> Self
    where
        Self: Sized;

    fn on_startup(&mut self, _world: &mut World)
    where
        Self: Sized,
    {
    }

    fn on_update(&mut self, _delta_time: f64, _world: &mut World)
    where
        Self: Sized,
    {
    }
}
