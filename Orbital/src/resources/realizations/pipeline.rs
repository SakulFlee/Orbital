use log::info;
use std::sync::{Mutex, OnceLock};
use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FragmentState, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor,
    StencilState, TextureFormat, VertexState,
};

use crate::{
    cache::Cache,
    error::Error,
    resources::{descriptors::PipelineDescriptor, realizations::Shader},
};

use super::{Instance, Vertex};

#[derive(Debug)]
pub struct Pipeline {
    render_pipeline: RenderPipeline,
    bind_group_layouts: Vec<(&'static str, BindGroupLayout)>,
    shader: Shader,
}

impl Pipeline {
    // --- Static ---
    /// Gives access to the internal pipeline cache.
    /// If the cache doesn't exist yet, it gets initialized.
    ///
    /// # Safety
    /// This is potentially a dangerous operation!
    /// The Rust compiler says the following:
    ///
    /// > use of mutable static is unsafe and requires unsafe function or block
    /// > mutable statics can be mutated by multiple threads: aliasing violations
    /// > or data races will cause undefined behavior
    ///
    /// However, once initialized, the cell [OnceLock] should never change and
    /// thus this should be safe.
    ///
    /// Additionally, we utilize a [Mutex] to ensure that access to the
    /// cache map and texture format is actually exclusive.
    pub unsafe fn cache() -> &'static mut (Cache<PipelineDescriptor, Pipeline>, TextureFormat) {
        static mut CACHE: OnceLock<Mutex<(Cache<PipelineDescriptor, Pipeline>, TextureFormat)>> =
            OnceLock::new();

        if CACHE.get().is_none() {
            info!("Pipeline cache doesn't exist! Initializing ...");
            let _ = CACHE.get_or_init(|| Mutex::new((Cache::new(), TextureFormat::Bgra8UnormSrgb)));
        }

        CACHE
            .get_mut()
            .unwrap()
            .get_mut()
            .expect("Cache access violation!")
    }

    /// Makes sure the cache is in the right state before accessing.
    /// Should be ideally called before each cache access.
    /// Once per context is enough though!
    ///
    /// This will set some cache parameters, if they don't exist yet
    /// (e.g. in case of a new cache), and make sure the pipelines
    /// still match the correct surface texture formats.
    /// If needed, this will also attempt recompiling all pipelines
    /// (and thus their shaders) to match a different format!
    pub fn prepare_cache_access(
        potential_new_format: Option<&TextureFormat>,
        device: &Device,
        queue: &Queue,
    ) -> &'static mut Cache<PipelineDescriptor, Pipeline> {
        let (cache, cache_format) = unsafe { Self::cache() };

        // Check the cache format if it's set to [Some]
        if let Some(potential_new_format) = potential_new_format {
            // If they don't match, trigger a recompilation of the cache!
            if cache_format != potential_new_format {
                // If the formats don't match, we have to attempt to
                // recompile the whole cache.

                // Thus, set the new cache format BEFORE cache is accessed again ...
                *cache_format = *potential_new_format;

                // ... and rework the cache!
                cache.rework(|k| Self::make_pipeline(k, potential_new_format, device, queue).ok());
            }
        }

        cache
    }

    // --- Constructor ---
    pub fn from_descriptor(
        pipeline_descriptor: &PipelineDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<&'static Self, Error> {
        let cache = Self::prepare_cache_access(Some(surface_format), device, queue);

        cache.get_or_add_fallible(pipeline_descriptor, |k| {
            Self::make_pipeline(k, surface_format, device, queue)
        })
    }

    fn make_pipeline(
        pipeline_descriptor: &PipelineDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Pipeline, Error> {
        let bind_group_layouts = pipeline_descriptor
            .bind_group_layouts
            .iter()
            .map(|x| (x.label, x.make_bind_group_layout(device)))
            .collect::<Vec<_>>();
        let bind_group_layouts_ref = bind_group_layouts
            .iter()
            .map(|(_, x)| x)
            .collect::<Vec<_>>();

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts_ref,
            push_constant_ranges: &[],
        });

        let shader = Shader::from_descriptor(pipeline_descriptor.shader_descriptor, device, queue)?;

        let mut vertex_buffers = Vec::new();
        if pipeline_descriptor.include_vertex_buffer_layout {
            vertex_buffers.push(Vertex::vertex_buffer_layout_descriptor());
        }
        if pipeline_descriptor.include_instance_buffer_layout {
            vertex_buffers.push(Instance::vertex_buffer_layout_descriptor());
        }

        let depth_stencil = if pipeline_descriptor.depth_stencil {
            Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            })
        } else {
            None
        };

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: shader.shader_module(),
                entry_point: "entrypoint_vertex",
                buffers: &vertex_buffers,
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: shader.shader_module(),
                entry_point: "entrypoint_fragment",
                targets: &[Some(ColorTargetState {
                    format: *surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: pipeline_descriptor.primitive_topology,
                strip_index_format: None,
                front_face: pipeline_descriptor.front_face_order,
                cull_mode: pipeline_descriptor.cull_mode,
                unclipped_depth: false,
                polygon_mode: pipeline_descriptor.polygon_mode,
                conservative: false,
            },
            depth_stencil,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            render_pipeline,
            bind_group_layouts,
            shader,
        })
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }

    pub fn bind_group_layout(&self, label: &str) -> Option<&BindGroupLayout> {
        self.bind_group_layouts
            .iter()
            .find(|x| x.0 == label)
            .map(|(_, x)| x)
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}
