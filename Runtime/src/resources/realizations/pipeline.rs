use log::error;
use std::{collections::HashMap, sync::OnceLock};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BlendState, ColorTargetState, ColorWrites, Device,
    FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor, TextureFormat, VertexState,
};

use crate::{
    error::Error,
    resources::{PipelineDescriptor, Shader, VertexUniform},
};

pub struct Pipeline {
    render_pipeline: RenderPipeline,
    bind_group_layout: BindGroupLayout,
    shader: Shader,
}

impl Pipeline {
    // --- Static ---
    pub unsafe fn cache() -> (
        &'static mut HashMap<PipelineDescriptor, Pipeline>,
        &'static mut Option<TextureFormat>,
    ) {
        static mut CACHE: OnceLock<HashMap<PipelineDescriptor, Pipeline>> = OnceLock::new();
        static mut CACHE_FORMAT: OnceLock<Option<TextureFormat>> = OnceLock::new();

        let cache_ref = match CACHE.get_mut() {
            Some(r) => r,
            None => {
                CACHE.set(HashMap::new());
                CACHE.get_mut().unwrap()
            }
        };
        let cache_format_ref = match CACHE_FORMAT.get_mut() {
            Some(r) => r,
            None => {
                CACHE_FORMAT.set(None);
                CACHE_FORMAT.get_mut().unwrap()
            }
        };

        (cache_ref, cache_format_ref)
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
        potential_new_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> &'static mut HashMap<PipelineDescriptor, Pipeline> {
        let (cache, cache_format) = unsafe { Self::cache() };

        // Check the cache format
        match cache_format {
            Some(current_format) => {
                // If there is a format, compare it.
                // If they don't match, trigger a recompilation of the cache!
                if current_format != potential_new_format {
                    // If the formats don't match, we have to attempt to
                    // recompile the whole cache.

                    // Thus, set the new cache format BEFORE cache is accessed
                    // again ...
                    *cache_format = Some(*potential_new_format);

                    // ... drain the map ...
                    let old_cache = cache.drain();

                    // ... loop through each descriptor (key) ...
                    for (descriptor, _) in old_cache {
                        // ... and create new pipelines with the new format!
                        // Keep in mind that making a pipeline from a descriptor
                        // will automatically add the pipeline to the cache.
                        // Thus, we don't have to add anything special here!
                        // However, errors are nasty:
                        if let Err(e) =
                            Self::from_descriptor(&descriptor, &potential_new_format, device, queue)
                        {
                            error!("Failed recompiling pipeline (and shader) in pipeline cache after surface format change! Error: {:?}", e);
                        }
                    }
                }
            }
            None => {
                // If no surface format is set yet, this cache must be new.
                // Fill it with the current format!
                *cache_format = Some(*potential_new_format);
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
        let cache = Self::prepare_cache_access(surface_format, device, queue);

        // Now that the cache is ready, check if we have a pipeline like
        // the descriptor one describes in cache.
        // If so, we can just return a reference to it.
        // Otherwise, we need to make it ...
        if cache.contains_key(pipeline_descriptor) {
            return Ok(cache.get(pipeline_descriptor).unwrap());
        } else {
            // Actually make the pipeline ...
            let pipeline = Self::make_pipeline(pipeline_descriptor, surface_format, device, queue)?;

            // ... insert it into the cache ...
            cache.insert(pipeline_descriptor.clone(), pipeline);

            // ... and return a reference to it!
            Ok(cache.get_mut(pipeline_descriptor).unwrap())
        }
    }

    fn make_pipeline(
        pipeline_descriptor: &PipelineDescriptor,
        surface_format: &TextureFormat,
        device: &Device,
        queue: &Queue,
    ) -> Result<Pipeline, Error> {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &pipeline_descriptor.bind_group_entries.as_slice(),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader =
            Shader::from_descriptor(&pipeline_descriptor.shader_descriptor, device, queue)?;

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: shader.vertex_shader_module(),
                entry_point: "main",
                buffers: &[VertexUniform::descriptor()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: shader.fragment_shader_module(),
                entry_point: "main",
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
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            render_pipeline,
            bind_group_layout,
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
