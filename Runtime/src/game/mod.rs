use crate::server::RenderServer;

pub mod runtime;
pub use runtime::*;

pub mod settings;
pub use settings::*;

pub trait Game {
    fn init() -> Self
    where
        Self: Sized;

    fn cycle(&mut self, delta_time: f64, render_server: &mut RenderServer)
    where
        Self: Sized;
}
