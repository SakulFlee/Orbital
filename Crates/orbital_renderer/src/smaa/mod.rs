//! SMAA 1x post-process anti-aliasing.
//!
//! WGSL adaptation of the MIT-licensed [wgsl-smaa] port of the SMAA reference
//! implementation by Jimenez et al. The three passes are recorded into the
//! caller's command encoder so the renderer keeps one scene submit per frame.
//!
//! [wgsl-smaa]: https://github.com/dmnsgn/shaders-smaa

use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, Buffer, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, Device, FragmentState, LoadOp, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDimension, VertexState, util::DeviceExt,
};

const AREATEX_WIDTH: u32 = 160;
const AREATEX_HEIGHT: u32 = 560;
const AREATEX_BYTES: &[u8] = include_bytes!("AreaTex.bin");

const SEARCHTEX_WIDTH: u32 = 64;
const SEARCHTEX_HEIGHT: u32 = 16;
const SEARCHTEX_BYTES: &[u8] = include_bytes!("SearchTex.bin");

const EDGES_FORMAT: TextureFormat = TextureFormat::Rg8Unorm;
const BLEND_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

/// SMAA quality preset. Controls edge threshold, search distance and whether
/// diagonal/corner handling is enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SmaaQuality {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

impl SmaaQuality {
    fn threshold(self) -> f64 {
        match self {
            Self::Low => 0.15,
            Self::Medium | Self::High => 0.1,
            Self::Ultra => 0.05,
        }
    }

    fn max_search_steps(self) -> f64 {
        match self {
            Self::Low => 4.0,
            Self::Medium => 8.0,
            Self::High => 16.0,
            Self::Ultra => 32.0,
        }
    }

    fn max_diagonal_steps(self) -> f64 {
        match self {
            Self::Low | Self::Medium => 0.0,
            Self::High => 8.0,
            Self::Ultra => 16.0,
        }
    }

    fn corner_rounding(self) -> f64 {
        match self {
            Self::Low | Self::Medium => 0.0,
            Self::High | Self::Ultra => 25.0,
        }
    }

    fn disable_diagonal(self) -> f64 {
        match self {
            Self::Low | Self::Medium => 1.0,
            Self::High | Self::Ultra => 0.0,
        }
    }

    fn disable_corner(self) -> f64 {
        match self {
            Self::Low | Self::Medium => 1.0,
            Self::High | Self::Ultra => 0.0,
        }
    }
}

struct BindGroupLayouts {
    edge: BindGroupLayout,
    weights: BindGroupLayout,
    blend: BindGroupLayout,
}

struct Pipelines {
    edge: RenderPipeline,
    weights: RenderPipeline,
    blend: RenderPipeline,
}

struct Resources {
    area_view: TextureView,
    search_view: TextureView,
    linear_sampler: Sampler,
    point_sampler: Sampler,
    subsample_indices: Buffer,
}

struct Targets {
    color: TextureView,
    edges: TextureView,
    blend: TextureView,
}

struct BindGroups {
    edge: BindGroup,
    weights: BindGroup,
    blend: BindGroup,
}

impl BindGroupLayouts {
    fn new(device: &Device) -> Self {
        let edge = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::SmaaEdgeLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
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
            ],
        });

        let opaque_texture = |binding: u32| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };

        let weights = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::SmaaWeightsLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                opaque_texture(1),
                opaque_texture(2),
                opaque_texture(3),
                BindGroupLayoutEntry {
                    binding: 4,
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

        let blend = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::SmaaBlendLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                opaque_texture(1),
                opaque_texture(2),
            ],
        });

        Self {
            edge,
            weights,
            blend,
        }
    }
}

impl Pipelines {
    fn new(
        device: &Device,
        format: TextureFormat,
        layouts: &BindGroupLayouts,
        quality: SmaaQuality,
    ) -> Self {
        let edge_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::SmaaEdgeShader"),
            source: ShaderSource::Wgsl(include_str!("edges.wgsl").into()),
        });
        let weights_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::SmaaWeightsShader"),
            source: ShaderSource::Wgsl(include_str!("weights.wgsl").into()),
        });
        let blend_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::SmaaBlendShader"),
            source: ShaderSource::Wgsl(include_str!("blend.wgsl").into()),
        });

        let edge_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::SmaaEdgePipelineLayout"),
            bind_group_layouts: &[Some(&layouts.edge)],
            immediate_size: 0,
        });
        let weights_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::SmaaWeightsPipelineLayout"),
            bind_group_layouts: &[Some(&layouts.weights)],
            immediate_size: 0,
        });
        let blend_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::SmaaBlendPipelineLayout"),
            bind_group_layouts: &[Some(&layouts.blend)],
            immediate_size: 0,
        });

        let edge_constants = [
            ("SMAA_SRGB_INPUT", if format.is_srgb() { 1.0 } else { 0.0 }),
            ("SMAA_THRESHOLD", quality.threshold()),
        ];
        let edge_compilation = PipelineCompilationOptions {
            constants: &edge_constants,
            ..Default::default()
        };
        let weights_constants = [
            ("SMAA_MAX_SEARCH_STEPS", quality.max_search_steps()),
            ("SMAA_MAX_SEARCH_STEPS_DIAG", quality.max_diagonal_steps()),
            ("SMAA_CORNER_ROUNDING", quality.corner_rounding()),
            ("SMAA_DISABLE_DIAG_DETECTION", quality.disable_diagonal()),
            ("SMAA_DISABLE_CORNER_DETECTION", quality.disable_corner()),
        ];
        let weights_compilation = PipelineCompilationOptions {
            constants: &weights_constants,
            ..Default::default()
        };

        let pipeline =
            |label: &'static str,
             layout: &wgpu::PipelineLayout,
             module: &wgpu::ShaderModule,
             target: TextureFormat,
             compilation_options: PipelineCompilationOptions<'_>| {
                device.create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(layout),
                    vertex: VertexState {
                        module,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(FragmentState {
                        module,
                        entry_point: Some("fs_main"),
                        targets: &[Some(ColorTargetState {
                            format: target,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })],
                        compilation_options,
                    }),
                    depth_stencil: None,
                    primitive: PrimitiveState::default(),
                    cache: None,
                    multisample: Default::default(),
                    multiview_mask: None,
                })
            };

        Self {
            edge: pipeline(
                "Orbital::SmaaEdgePipeline",
                &edge_layout,
                &edge_shader,
                EDGES_FORMAT,
                edge_compilation,
            ),
            weights: pipeline(
                "Orbital::SmaaWeightsPipeline",
                &weights_layout,
                &weights_shader,
                BLEND_FORMAT,
                weights_compilation,
            ),
            blend: pipeline(
                "Orbital::SmaaBlendPipeline",
                &blend_layout,
                &blend_shader,
                format,
                Default::default(),
            ),
        }
    }
}

impl Resources {
    fn new(device: &Device, queue: &Queue) -> Self {
        let area_texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Orbital::SmaaAreaTexture"),
                size: wgpu::Extent3d {
                    width: AREATEX_WIDTH,
                    height: AREATEX_HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rg8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            AREATEX_BYTES,
        );
        let search_texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("Orbital::SmaaSearchTexture"),
                size: wgpu::Extent3d {
                    width: SEARCHTEX_WIDTH,
                    height: SEARCHTEX_HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::R8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            SEARCHTEX_BYTES,
        );

        let linear_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Orbital::SmaaLinearSampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let point_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Orbital::SmaaPointSampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let subsample_indices = device.create_buffer(&BufferDescriptor {
            label: Some("Orbital::SmaaSubsampleIndices"),
            size: 16,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&subsample_indices, 0, &[0u8; 16]);

        Self {
            area_view: area_texture.create_view(&Default::default()),
            search_view: search_texture.create_view(&Default::default()),
            linear_sampler,
            point_sampler,
            subsample_indices,
        }
    }
}

impl Targets {
    fn new(device: &Device, width: u32, height: u32, format: TextureFormat) -> Self {
        let target = |label: &'static str, format: TextureFormat| {
            device
                .create_texture(&TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format,
                    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };

        Self {
            color: target("Orbital::SmaaColorTarget", format),
            edges: target("Orbital::SmaaEdgesTarget", EDGES_FORMAT),
            blend: target("Orbital::SmaaBlendTarget", BLEND_FORMAT),
        }
    }
}

impl BindGroups {
    fn new(
        device: &Device,
        layouts: &BindGroupLayouts,
        resources: &Resources,
        targets: &Targets,
    ) -> Self {
        let edge = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Orbital::SmaaEdgeBindGroup"),
            layout: &layouts.edge,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&resources.point_sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&targets.color),
                },
            ],
        });
        let weights = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Orbital::SmaaWeightsBindGroup"),
            layout: &layouts.weights,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&resources.linear_sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&targets.edges),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&resources.area_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&resources.search_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: resources.subsample_indices.as_entire_binding(),
                },
            ],
        });
        let blend = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Orbital::SmaaBlendBindGroup"),
            layout: &layouts.blend,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Sampler(&resources.linear_sampler),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&targets.color),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&targets.blend),
                },
            ],
        });

        Self {
            edge,
            weights,
            blend,
        }
    }
}

struct SmaaTargetInner {
    pipelines: Pipelines,
    layouts: BindGroupLayouts,
    resources: Resources,
    targets: Targets,
    bind_groups: BindGroups,
    format: TextureFormat,
}

/// Owns the SMAA intermediate targets and records the three SMAA passes.
pub struct SmaaTarget {
    inner: SmaaTargetInner,
}

impl SmaaTarget {
    /// Create a new SMAA target.
    pub fn new(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        format: TextureFormat,
        quality: SmaaQuality,
    ) -> Self {
        let layouts = BindGroupLayouts::new(device);
        let pipelines = Pipelines::new(device, format, &layouts, quality);
        let resources = Resources::new(device, queue);
        let targets = Targets::new(device, width, height, format);
        let bind_groups = BindGroups::new(device, &layouts, &resources, &targets);

        Self {
            inner: SmaaTargetInner {
                pipelines,
                layouts,
                resources,
                targets,
                bind_groups,
                format,
            },
        }
    }

    /// Resize the SMAA targets.
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.inner.targets = Targets::new(device, width, height, self.inner.format);
        self.inner.bind_groups = BindGroups::new(
            device,
            &self.inner.layouts,
            &self.inner.resources,
            &self.inner.targets,
        );
    }

    /// The LDR color target the scene should tonemap into.
    pub fn color_target(&self) -> &TextureView {
        &self.inner.targets.color
    }

    /// Record the edge, weight, and neighborhood-blending passes.
    pub fn record(&self, encoder: &mut wgpu::CommandEncoder, output_view: &TextureView) {
        let inner = &self.inner;

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::SmaaEdges"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &inner.targets.edges,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&inner.pipelines.edge);
        pass.set_bind_group(0, &inner.bind_groups.edge, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::SmaaWeights"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &inner.targets.blend,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&inner.pipelines.weights);
        pass.set_bind_group(0, &inner.bind_groups.weights, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::SmaaBlend"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&inner.pipelines.blend);
        pass.set_bind_group(0, &inner.bind_groups.blend, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smaa_wgsl_sources_are_valid() {
        for (label, source) in [
            ("edges", include_str!("edges.wgsl")),
            ("weights", include_str!("weights.wgsl")),
            ("blend", include_str!("blend.wgsl")),
        ] {
            let module = wgpu::naga::front::wgsl::parse_str(source)
                .unwrap_or_else(|error| panic!("SMAA {label} shader failed to parse: {error:?}"));
            let mut validator = wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::all(),
            );
            validator
                .validate(&module)
                .unwrap_or_else(|error| panic!("SMAA {label} shader failed validation: {error:?}"));
        }
    }

    #[test]
    fn smaa_pipelines_create_and_validate_on_noop_backend() {
        let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
        let output = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Smaa test output"),
                size: wgpu::Extent3d {
                    width: 320,
                    height: 180,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default());

        for quality in [
            super::SmaaQuality::Low,
            super::SmaaQuality::Medium,
            super::SmaaQuality::High,
            super::SmaaQuality::Ultra,
        ] {
            let target = super::SmaaTarget::new(
                &device,
                &queue,
                320,
                180,
                wgpu::TextureFormat::Bgra8UnormSrgb,
                quality,
            );
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Smaa test encoder"),
            });
            target.record(&mut encoder, &output);
            queue.submit([encoder.finish()]);
        }
    }
}
