use wgpu::{SurfaceConfiguration, TextureView};

use crate::runtime::Context;

pub trait App {
    fn init(config: &SurfaceConfiguration, context: &Context) -> Self
    where
        Self: Sized;

    fn resize(&mut self, config: &SurfaceConfiguration, context: &Context);

    fn update(&mut self);

    fn render(&mut self, view: &TextureView, context: &Context);
}
