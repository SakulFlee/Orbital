//! Froxel volumetric lighting and height fog.

use std::sync::Arc;

use cgmath::Vector2;
use orbital_ecs_bridge::{VolumetricQuality, VolumetricSettings};
use orbital_shader_preprocessor::{NodeRegistry, ShaderBuilder};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, Buffer, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, FragmentState, LoadOp, Operations, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, StoreOp,
    TextureSampleType, TextureView, TextureViewDimension, VertexState,
};

const INJECT_NODES: &[&str] = &[
    "camera_uniform",
    "light_struct",
    "light_types",
    "shadow_types",
    "shadow_slot",
    "slope_scaled_bias",
    "spot_bias_scale",
    "sample_shadow_2d_pcf",
    "compute_shadow_for_light",
];

const INJECT_PARAMS_SIZE: u64 = 64;
const COMPOSITE_PARAMS_SIZE: u64 = 16;

/// Keep the world-space spacing between froxel depth slices small enough
/// that short light cones do not pop between slices as the camera moves.
const TARGET_DEPTH_SLICE_SPACING: f32 = 0.15;
/// Upper bound for adaptive depth slices; the X/Y grid still follows the
/// selected `VolumetricQuality`.
const MAX_DEPTH_SLICES: u32 = 192;

pub struct VolumetricRenderer {
    inject_pipeline: ComputePipeline,
    inject_layout: BindGroupLayout,
    composite_pipeline: RenderPipeline,
    composite_layout: BindGroupLayout,
    froxel_sampler: Sampler,
    inject_params: Buffer,
    composite_params: Buffer,
    grid: [u32; 3],
    steps: u32,
    inject_bind_group: Option<(TextureView, BindGroup)>,
    composite_bind_group: Option<CompositeBindGroupCache>,
}

struct CompositeBindGroupCache {
    scene_color: TextureView,
    depth: TextureView,
    froxel: TextureView,
    bind_group: BindGroup,
}

impl VolumetricRenderer {
    pub fn new(
        device: &Device,
        queue: &Queue,
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
            .add_nodes(INJECT_NODES)
            .expect("volumetric inject nodes");
        builder.add_source(include_str!("volumetric_inject.wgsl"));
        let inject_source = builder.build();
        let inject_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::VolumetricInjectShader"),
            source: ShaderSource::Wgsl(inject_source.into()),
        });

        let mut composite_builder = ShaderBuilder::new(Arc::new({
            let mut registry = NodeRegistry::new();
            registry
                .register_library(&orbital_shader_math::math_library())
                .expect("math node library");
            registry
                .register_library(&orbital_shader_engine::engine_library())
                .expect("engine node library");
            registry
        }));
        composite_builder
            .add_nodes(&["camera_uniform"])
            .expect("volumetric composite nodes");
        composite_builder.add_source(include_str!("volumetric_composite.wgsl"));
        let composite_source = composite_builder.build();
        let composite_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::VolumetricCompositeShader"),
            source: ShaderSource::Wgsl(composite_source.into()),
        });

        let inject_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::VolumetricInjectLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D3,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let inject_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::VolumetricInjectPipelineLayout"),
            bind_group_layouts: &[Some(world_layout), Some(&inject_layout)],
            immediate_size: 0,
        });

        let inject_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Orbital::VolumetricInjectPipeline"),
            layout: Some(&inject_pipeline_layout),
            module: &inject_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let composite_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::VolumetricCompositeLayout"),
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
                        sample_type: TextureSampleType::Depth,
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
                        view_dimension: TextureViewDimension::D3,
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

        let composite_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::VolumetricCompositePipelineLayout"),
            bind_group_layouts: &[Some(world_layout), Some(&composite_layout)],
            immediate_size: 0,
        });

        let targets = [Some(ColorTargetState {
            format: target_format,
            blend: Some(BlendState::REPLACE),
            write_mask: ColorWrites::ALL,
        })];

        let composite_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Orbital::VolumetricCompositePipeline"),
            layout: Some(&composite_pipeline_layout),
            vertex: VertexState {
                module: &composite_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &composite_module,
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

        let froxel_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Orbital::VolumetricFroxelSampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let inject_params = device.create_buffer(&BufferDescriptor {
            label: Some("Orbital::VolumetricInjectParams"),
            size: INJECT_PARAMS_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let composite_params = device.create_buffer(&BufferDescriptor {
            label: Some("Orbital::VolumetricCompositeParams"),
            size: COMPOSITE_PARAMS_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut renderer = Self {
            inject_pipeline,
            inject_layout,
            composite_pipeline,
            composite_layout,
            froxel_sampler,
            inject_params,
            composite_params,
            grid: [1, 1, 1],
            steps: 1,
            inject_bind_group: None,
            composite_bind_group: None,
        };
        renderer.prepare(queue, &VolumetricSettings::default(), Vector2::new(1, 1));
        renderer
    }

    pub fn prepare(
        &mut self,
        queue: &Queue,
        settings: &VolumetricSettings,
        resolution: Vector2<u32>,
    ) {
        let mut grid = settings.quality.grid(resolution.x, resolution.y);
        if !matches!(settings.quality, VolumetricQuality::Custom { .. }) {
            // Fixed slice counts make a longer max distance coarser in world
            // space. Scale the depth resolution with the requested range so
            // short spot cones keep the same sampling density.
            let target =
                (settings.max_distance.max(0.1) / TARGET_DEPTH_SLICE_SPACING).ceil() as u32;
            grid[2] = grid[2].max(target);
        }
        self.grid = [
            grid[0].clamp(1, 512),
            grid[1].clamp(1, 512),
            grid[2].clamp(1, MAX_DEPTH_SLICES),
        ];
        self.steps = settings.quality.composite_steps().max(1);

        let mut inject = [0u8; INJECT_PARAMS_SIZE as usize];
        for (index, value) in self.grid.iter().enumerate() {
            inject[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        let media = [
            settings.density,
            settings.scattering,
            settings.extinction,
            settings.anisotropy,
        ];
        for (index, value) in media.iter().enumerate() {
            let offset = 16 + index * 4;
            inject[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        let height = [
            settings.height_base,
            settings.height_falloff,
            settings.height_density,
            settings.max_distance,
        ];
        for (index, value) in height.iter().enumerate() {
            let offset = 32 + index * 4;
            inject[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (index, value) in settings.ambient_color.iter().enumerate() {
            let offset = 48 + index * 4;
            inject[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        // `ambient.w` carries the environment/IBL contribution multiplier.
        inject[60..64].copy_from_slice(&settings.environment_intensity.to_le_bytes());
        queue.write_buffer(&self.inject_params, 0, &inject);

        let mut composite = [0u8; COMPOSITE_PARAMS_SIZE as usize];
        composite[0..4].copy_from_slice(&self.steps.to_le_bytes());
        composite[4..8].copy_from_slice(&settings.max_distance.to_le_bytes());
        queue.write_buffer(&self.composite_params, 0, &composite);
    }

    pub fn grid(&self) -> [u32; 3] {
        self.grid
    }

    pub fn record_inject(
        &mut self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        world_bind_group: &BindGroup,
        froxel_view: &TextureView,
    ) {
        let cache_hit = self
            .inject_bind_group
            .as_ref()
            .is_some_and(|(view, _)| view == froxel_view);

        if !cache_hit {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Orbital::VolumetricInjectBindGroup"),
                layout: &self.inject_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(froxel_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: self.inject_params.as_entire_binding(),
                    },
                ],
            });
            self.inject_bind_group = Some((froxel_view.clone(), bind_group));
        }

        let bind_group = &self
            .inject_bind_group
            .as_ref()
            .expect("volumetric inject bind group cache")
            .1;

        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Volumetric Inject"),
            ..Default::default()
        });
        pass.set_pipeline(&self.inject_pipeline);
        pass.set_bind_group(0, world_bind_group, &[]);
        pass.set_bind_group(1, bind_group, &[]);
        pass.dispatch_workgroups(
            self.grid[0].div_ceil(4),
            self.grid[1].div_ceil(4),
            self.grid[2].div_ceil(4),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_composite(
        &mut self,
        device: &Device,
        encoder: &mut wgpu::CommandEncoder,
        world_bind_group: &BindGroup,
        scene_color: &TextureView,
        depth: &TextureView,
        froxel: &TextureView,
        target: &TextureView,
    ) {
        let cache_hit = self.composite_bind_group.as_ref().is_some_and(|cache| {
            cache.scene_color == *scene_color && cache.depth == *depth && cache.froxel == *froxel
        });
        if !cache_hit {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Orbital::VolumetricCompositeBindGroup"),
                layout: &self.composite_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(scene_color),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(depth),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::TextureView(froxel),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::Sampler(&self.froxel_sampler),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: self.composite_params.as_entire_binding(),
                    },
                ],
            });
            self.composite_bind_group = Some(CompositeBindGroupCache {
                scene_color: scene_color.clone(),
                depth: depth.clone(),
                froxel: froxel.clone(),
                bind_group,
            });
        }
        let bind_group = &self
            .composite_bind_group
            .as_ref()
            .expect("volumetric composite bind group cache")
            .bind_group;

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::VolumetricComposite"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
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
        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(0, world_bind_group, &[]);
        pass.set_bind_group(1, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
