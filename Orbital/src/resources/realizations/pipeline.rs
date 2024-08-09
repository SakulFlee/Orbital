use log::info;
use std::sync::{Mutex, OnceLock};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device,
    FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    ShaderStages, StencilState, TextureFormat, TextureSampleType, TextureViewDimension,
    VertexState,
};

use crate::{
    cache::Cache,
    error::Error,
    resources::{descriptors::PipelineDescriptor, realizations::Shader},
};

use super::{Camera, Instance, LightStorage, Vertex};

#[derive(Debug)]
pub struct Pipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
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
    /// mutable statics can be mutated by multiple threads: aliasing violations
    /// or data races will cause undefined behavior
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
        let pipeline_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Normal
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Albedo
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Metallic
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Roughness
                    BindGroupLayoutEntry {
                        binding: 6,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 7,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                    // Occlusion
                    BindGroupLayoutEntry {
                        binding: 8,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 9,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                // Pipeline bind group layouts
                &pipeline_bind_group_layout,
                // Camera bind group layout
                &device.create_bind_group_layout(&Camera::bind_group_layout_descriptor()),
                // Light Storage bind group layout
                &device.create_bind_group_layout(&LightStorage::bind_group_layout_descriptor()),
            ],
            push_constant_ranges: &[],
        });

        let shader = Shader::from_descriptor(pipeline_descriptor.shader_descriptor, device, queue)?;

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: shader.shader_module(),
                entry_point: "entrypoint_vertex",
                buffers: &[
                    Vertex::vertex_buffer_layout_descriptor(),
                    Instance::vertex_buffer_layout_descriptor(),
                ],
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
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            render_pipeline,
            bind_group_layout: pipeline_bind_group_layout,
            shader,
        })
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}
