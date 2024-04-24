use wgpu::{RenderPipeline, TextureFormat};

use crate::{
    resources::PipelineBuilder,
    runtime::Context,
    shader::{Shader, ShaderType},
};

use super::Material;

pub struct DummyMaterial;

impl Material for DummyMaterial {
    fn render_pipeline_identifier(&self) -> &'static str {
        "empty"
    }

    fn make_render_pipeline(
        &self,
        context: &Context,
        surface_texture_format: Option<TextureFormat>,
    ) -> RenderPipeline {
        PipelineBuilder::new(
            Shader::from_file(
                "Shaders/Materials/Dummy.wgsl",
                ShaderType::WGSL(Some("Dummy")),
                "main_vs".into(),
                Some("main_fs".into()),
                context,
            )
            .expect("Shader failure"),
        )
        .build(context, surface_texture_format)
    }
}
