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

    fn cycle(&mut self, delta_time: f64, world: &mut World)
    where
        Self: Sized;
}
