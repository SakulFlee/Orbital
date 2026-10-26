//! Deferred lighting fullscreen pass.

use std::sync::Arc;

use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, ColorTargetState, ColorWrites, Device, FragmentState,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    StoreOp, TextureSampleType, TextureView, TextureViewDimension, VertexState,
};

use orbital_shader_pbr::{DEFERRED_LIGHTING_NODES, DEFERRED_LIGHTING_WGSL};
use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};

pub struct DeferredLighting {
    pipeline: RenderPipeline,
    gbuffer_layout: BindGroupLayout,
    bind_group_cache: Option<GbufferBindGroupCache>,
}

struct GbufferBindGroupCache {
    albedo: TextureView,
    normal: TextureView,
    metallic: TextureView,
    depth: TextureView,
    counts: wgpu::Buffer,
    indices: wgpu::Buffer,
    params: wgpu::Buffer,
    bind_group: BindGroup,
}

impl DeferredLighting {
    pub fn new(
        device: &Device,
        target_format: wgpu::TextureFormat,
        world_layout: &BindGroupLayout,
    ) -> Self {
        let mut registry = NodeRegistry::new();
        registry
            .register_library(&orbital_shader_math::math_library())
            .expect("math node library");
        registry
            .register_library(&orbital_shader_engine::engine_library())
            .expect("engine node library");
        registry
            .register_library(&orbital_shader_pbr::pbr_library())
            .expect("PBR node library");

        let mut builder = ShaderBuilder::new(Arc::new(registry));
        builder
            .add_nodes(DEFERRED_LIGHTING_NODES)
            .expect("deferred lighting nodes");
        builder.add_source(DEFERRED_LIGHTING_WGSL);
        let source = builder.build();

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::DeferredLightingShader"),
            source: ShaderSource::Wgsl(source.into()),
        });

        let gbuffer_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::GbufferLayout"),
            entries: &[
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
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
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
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::DeferredLightingPipelineLayout"),
            bind_group_layouts: &[Some(world_layout), Some(&gbuffer_layout)],
            immediate_size: 0,
        });

        let targets = [Some(ColorTargetState {
            format: target_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Orbital::DeferredLightingPipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &targets,
                compilation_options: Default::default(),
            }),
            depth_stencil: None,
            primitive: PrimitiveState::default(),
            cache: None,
            multisample: Default::default(),
            multiview_mask: None,
        });

        Self {
            pipeline,
            gbuffer_layout,
            bind_group_cache: None,
        }
    }

    /// Drop cached bind groups. Call when the deferred path is not in use so
    /// pooled render targets are not kept alive by the cache.
    pub fn clear_cache(&mut self) {
        self.bind_group_cache = None;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        world_bind_group: &BindGroup,
        gbuffer_albedo: &TextureView,
        gbuffer_normal: &TextureView,
        gbuffer_metallic: &TextureView,
        depth_view: &TextureView,
        tile_counts: &wgpu::Buffer,
        tile_indices: &wgpu::Buffer,
        tile_params: &wgpu::Buffer,
        target_view: &TextureView,
    ) {
        let cache_hit = self.bind_group_cache.as_ref().is_some_and(|cache| {
            cache.albedo == *gbuffer_albedo
                && cache.normal == *gbuffer_normal
                && cache.metallic == *gbuffer_metallic
                && cache.depth == *depth_view
                && cache.counts == *tile_counts
                && cache.indices == *tile_indices
                && cache.params == *tile_params
        });

        if !cache_hit {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Orbital::GbufferBindGroup"),
                layout: &self.gbuffer_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(gbuffer_albedo),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(gbuffer_normal),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(gbuffer_metallic),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(depth_view),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: tile_counts.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 5,
                        resource: tile_indices.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 6,
                        resource: tile_params.as_entire_binding(),
                    },
                ],
            });
            self.bind_group_cache = Some(GbufferBindGroupCache {
                albedo: gbuffer_albedo.clone(),
                normal: gbuffer_normal.clone(),
                metallic: gbuffer_metallic.clone(),
                depth: depth_view.clone(),
                counts: tile_counts.clone(),
                indices: tile_indices.clone(),
                params: tile_params.clone(),
                bind_group,
            });
        }

        let bind_group = &self
            .bind_group_cache
            .as_ref()
            .expect("G-buffer bind group cache was filled")
            .bind_group;

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::DeferredLighting"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, world_bind_group, &[]);
        pass.set_bind_group(1, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
