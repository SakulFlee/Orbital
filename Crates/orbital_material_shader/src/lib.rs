use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub use orbital_shader_core::{ShaderDescriptor, ShaderError, Variables};
use wgpu::{BindGroup, Device, Queue, RenderPipeline, TextureFormat};

mod world_bind_group;
pub use world_bind_group::make_world_bind_group_layout;

mod descriptor;
pub use descriptor::*;

mod cache;
pub use cache::*;

mod program;
pub use program::*;

mod pipeline_cache;
pub use pipeline_cache::*;

static PIPELINES_CREATED: AtomicU64 = AtomicU64::new(0);

/// Returns the cumulative number of material render pipelines created.
pub fn pipelines_created() -> u64 {
    PIPELINES_CREATED.load(Ordering::Relaxed)
}

mod vertex_stage_layout;
pub use vertex_stage_layout::*;

/// A realized material instance: a compiled shader program plus its bind group
/// and a cached forward pipeline.
#[derive(Debug)]
pub struct MaterialShader {
    program: Arc<ShaderProgram>,
    pipeline: Arc<RenderPipeline>,
    gbuffer_pipeline: Option<Arc<RenderPipeline>>,
    bind_group: Option<BindGroup>,
    variables: Option<Variables>,
}

impl MaterialShader {
    /// Creates a realized material without sharing a pipeline cache.
    ///
    /// Prefer [`MaterialShader::from_descriptor_with_cache`] when a
    /// device-wide cache is available.
    pub fn from_descriptor(
        descriptor: &MaterialShaderDescriptor,
        surface_format: Option<TextureFormat>,
        device: &Device,
        queue: &Queue,
    ) -> Result<Self, Box<dyn Error>> {
        let mut pipeline_cache = PipelineCache::default();
        Self::from_descriptor_with_cache(
            descriptor,
            surface_format,
            device,
            queue,
            &mut pipeline_cache,
            None,
        )
    }

    /// Creates a realized material using a shared pipeline cache.
    pub fn from_descriptor_with_cache(
        descriptor: &MaterialShaderDescriptor,
        surface_format: Option<TextureFormat>,
        device: &Device,
        queue: &Queue,
        pipeline_cache: &mut PipelineCache,
        gbuffer_color_formats: Option<&[TextureFormat]>,
    ) -> Result<Self, Box<dyn Error>> {
        let color_format = surface_format.unwrap_or(TextureFormat::Rgba8UnormSrgb);
        let program = Arc::new(ShaderProgram::from_descriptor(descriptor, device)?);
        let variables = descriptor.realize_variables(device, queue)?;

        let bind_group = if let (Some(layout), Some(variables)) =
            (program.material_bind_group_layout(), variables.as_ref())
        {
            descriptor.bind_group_with_layout(layout, variables, device)?
        } else {
            None
        };

        let pipeline = pipeline_cache.forward_pipeline(&program, color_format, device);
        let gbuffer_pipeline = gbuffer_color_formats
            .and_then(|formats| pipeline_cache.gbuffer_pipeline(&program, formats, device));

        Ok(Self {
            program,
            pipeline,
            gbuffer_pipeline,
            bind_group,
            variables,
        })
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn gbuffer_pipeline(&self) -> Option<&RenderPipeline> {
        self.gbuffer_pipeline.as_deref()
    }

    pub fn program(&self) -> &ShaderProgram {
        &self.program
    }

    pub fn program_arc(&self) -> &Arc<ShaderProgram> {
        &self.program
    }

    pub fn has_shadow_variant(&self) -> bool {
        self.program.has_shadow_variant()
    }

    pub fn create_shadow_pipeline(
        &self,
        light_matrix_layout: &wgpu::BindGroupLayout,
        device: &Device,
    ) -> Option<RenderPipeline> {
        self.program
            .create_shadow_pipeline(light_matrix_layout, device)
    }

    pub fn alpha_mode(&self) -> AlphaMode {
        self.program.alpha_mode()
    }

    pub fn render_queue(&self) -> RenderQueue {
        self.program.render_queue()
    }

    pub fn cast_shadows(&self) -> bool {
        self.program.cast_shadows()
    }

    pub fn receive_shadows(&self) -> bool {
        self.program.receive_shadows()
    }

    pub fn bind_group(&self) -> Option<&BindGroup> {
        self.bind_group.as_ref()
    }

    pub fn variables(&self) -> Option<&Variables> {
        self.variables.as_ref()
    }
}
