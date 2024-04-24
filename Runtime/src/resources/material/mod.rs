use crate::runtime::Context;
use wgpu::{RenderPipeline, TextureFormat};

pub mod dummy;
pub use dummy::*;

pub trait Material {
    fn render_pipeline_identifier(&self) -> &'static str;
    fn make_render_pipeline(
        &self,
        context: &Context,
        surface_texture_format: Option<TextureFormat>,
    ) -> RenderPipeline;
    // TODO: Bindings
    // TODO: Uniforms
}
