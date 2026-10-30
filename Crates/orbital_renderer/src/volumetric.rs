//! Froxel volumetric lighting and height fog.

use std::sync::Arc;

use cgmath::{Matrix4, Vector2};
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
    TextureDescriptor, TextureDimension, TextureSampleType, TextureUsages, TextureView,
    TextureViewDimension, VertexState,
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
const ACCUM_PARAMS_SIZE: u64 = 160;

/// Temporal blend weight for valid history. Volumetric fog is low-frequency;
/// a high history weight removes the per-frame froxel shimmer without needing
/// a full-screen TAA pass.
const HISTORY_BLEND: f32 = 0.9;

/// Keep the world-space spacing between froxel depth slices small enough
/// that short light cones do not pop between slices as the camera moves.
const TARGET_DEPTH_SLICE_SPACING: f32 = 0.15;
/// Upper bound for adaptive depth slices; the X/Y grid still follows the
/// selected `VolumetricQuality`.
const MAX_DEPTH_SLICES: u32 = 192;

pub struct VolumetricRenderer {
    inject_pipeline: ComputePipeline,
    inject_layout: BindGroupLayout,
    accumulate_pipeline: ComputePipeline,
    accumulate_layout: BindGroupLayout,
    composite_pipeline: RenderPipeline,
    composite_layout: BindGroupLayout,
    froxel_sampler: Sampler,
    inject_params: Buffer,
    accum_params: Buffer,
    composite_params: Buffer,
    grid: [u32; 3],
    steps: u32,
    inject_bind_group: Option<(TextureView, BindGroup)>,
    composite_bind_group: Option<CompositeBindGroupCache>,
    history: Option<HistoryTargets>,
    history_index: usize,
    history_valid: bool,
    history_grid: Option<[u32; 3]>,
    history_resolution: Option<Vector2<u32>>,
    current_view: Option<Matrix4<f32>>,
    current_view_projection: Option<Matrix4<f32>>,
    previous_view: Option<Matrix4<f32>>,
    previous_view_projection: Option<Matrix4<f32>>,
    frame_index: u32,
    current_max_distance: f32,
    previous_max_distance: f32,
    temporal_enabled: bool,
}

struct HistoryTargets {
    _textures: [wgpu::Texture; 2],
    views: [TextureView; 2],
}

impl HistoryTargets {
    fn new(device: &Device, grid: [u32; 3]) -> Self {
        let make = |label: &'static str| {
            let texture = device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: grid[0],
                    height: grid[1],
                    depth_or_array_layers: grid[2],
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D3,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&Default::default());
            (texture, view)
        };
        let (texture0, view0) = make("Orbital::VolumetricHistory0");
        let (texture1, view1) = make("Orbital::VolumetricHistory1");
        Self {
            _textures: [texture0, texture1],
            views: [view0, view1],
        }
    }
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

        let mut accumulate_builder = ShaderBuilder::new(Arc::new({
            let mut registry = NodeRegistry::new();
            registry
                .register_library(&orbital_shader_math::math_library())
                .expect("math node library");
            registry
                .register_library(&orbital_shader_engine::engine_library())
                .expect("engine node library");
            registry
        }));
        accumulate_builder
            .add_nodes(&["camera_uniform"])
            .expect("volumetric accumulate nodes");
        accumulate_builder.add_source(include_str!("volumetric_accumulate.wgsl"));
        let accumulate_source = accumulate_builder.build();
        let accumulate_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Orbital::VolumetricAccumulateShader"),
            source: ShaderSource::Wgsl(accumulate_source.into()),
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

        let accumulate_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Orbital::VolumetricAccumulateLayout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D3,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
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

        let accumulate_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Orbital::VolumetricAccumulatePipelineLayout"),
            bind_group_layouts: &[Some(world_layout), Some(&accumulate_layout)],
            immediate_size: 0,
        });

        let accumulate_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Orbital::VolumetricAccumulatePipeline"),
            layout: Some(&accumulate_pipeline_layout),
            module: &accumulate_module,
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
        let accum_params = device.create_buffer(&BufferDescriptor {
            label: Some("Orbital::VolumetricAccumulateParams"),
            size: ACCUM_PARAMS_SIZE,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut renderer = Self {
            inject_pipeline,
            inject_layout,
            accumulate_pipeline,
            accumulate_layout,
            composite_pipeline,
            composite_layout,
            froxel_sampler,
            inject_params,
            accum_params,
            composite_params,
            grid: [1, 1, 1],
            steps: 1,
            inject_bind_group: None,
            composite_bind_group: None,
            history: None,
            history_index: 0,
            history_valid: false,
            history_grid: None,
            history_resolution: None,
            current_view: None,
            current_view_projection: None,
            previous_view: None,
            previous_view_projection: None,
            frame_index: 0,
            current_max_distance: 1.0,
            previous_max_distance: 0.0,
            temporal_enabled: true,
        };
        renderer.prepare(
            queue,
            &VolumetricSettings::default(),
            Vector2::new(1, 1),
            0,
            None,
            None,
        );
        renderer
    }

    /// Drop temporal history when volumetrics are disabled or invalidated by
    /// an external event such as a camera cut.
    pub fn reset_history(&mut self) {
        self.history = None;
        self.history_index = 0;
        self.history_valid = false;
        self.history_grid = None;
        self.history_resolution = None;
        self.previous_view = None;
        self.previous_view_projection = None;
        self.previous_max_distance = 0.0;
    }

    pub fn prepare(
        &mut self,
        queue: &Queue,
        settings: &VolumetricSettings,
        resolution: Vector2<u32>,
        frame_index: u64,
        current_view: Option<Matrix4<f32>>,
        current_view_projection: Option<Matrix4<f32>>,
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
        self.current_max_distance = settings.max_distance.max(0.1);
        self.frame_index = frame_index as u32;
        self.current_view = current_view;
        self.current_view_projection = current_view_projection;
        self.temporal_enabled = settings.temporal_history;
        if !self.temporal_enabled {
            self.reset_history();
        }

        let camera_cut = match (self.previous_view, current_view) {
            (Some(previous), Some(current)) => matrix_max_delta(&previous, &current) > 1.0,
            _ => false,
        };
        let history_invalid = !self.temporal_enabled
            || self.history_grid != Some(self.grid)
            || self.history_resolution != Some(resolution)
            || (self.current_max_distance - self.previous_max_distance).abs() > 1.0e-3
            || current_view.is_none()
            || current_view_projection.is_none()
            || camera_cut;
        if history_invalid {
            self.history = None;
            self.history_index = 0;
            self.history_valid = false;
            self.previous_view = None;
            self.previous_view_projection = None;
        }
        self.history_grid = Some(self.grid);
        self.history_resolution = Some(resolution);

        let mut inject = [0u8; INJECT_PARAMS_SIZE as usize];
        for (index, value) in self.grid.iter().enumerate() {
            inject[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        inject[12..16].copy_from_slice(&self.frame_index.to_le_bytes());
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

    /// Accumulate the current froxel volume with reprojected history.
    ///
    /// Returns the newly accumulated texture view, or `None` when the current
    /// camera is not available (in which case the caller should use the
    /// current froxel volume directly).
    pub fn record_accumulate(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut wgpu::CommandEncoder,
        world_bind_group: &BindGroup,
        current_froxels: &TextureView,
    ) -> Option<TextureView> {
        if !self.temporal_enabled {
            return None;
        }
        let current_view = self.current_view?;
        let current_view_projection = self.current_view_projection?;

        if self.history.is_none() {
            self.history = Some(HistoryTargets::new(device, self.grid));
            self.history_index = 0;
            self.history_valid = false;
        }

        let write_index = self.history_index;
        let read_index = 1 - write_index;
        let history_views = &self.history.as_ref()?.views;
        let history_view = &history_views[read_index];
        let output_view = &history_views[write_index];

        let history_valid = self.history_valid
            && self.previous_view.is_some()
            && self.previous_view_projection.is_some();
        let previous_view = self.previous_view.unwrap_or(current_view);
        let previous_view_projection = self
            .previous_view_projection
            .unwrap_or(current_view_projection);

        let mut params = [0u8; ACCUM_PARAMS_SIZE as usize];
        params[0..64].copy_from_slice(&matrix_to_bytes(&previous_view));
        params[64..128].copy_from_slice(&matrix_to_bytes(&previous_view_projection));
        for (index, value) in self.grid.iter().enumerate() {
            let offset = 128 + index * 4;
            params[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        params[144..148].copy_from_slice(&self.current_max_distance.to_le_bytes());
        params[148..152].copy_from_slice(&self.previous_max_distance.to_le_bytes());
        params[152..156].copy_from_slice(&HISTORY_BLEND.to_le_bytes());
        params[156..160]
            .copy_from_slice(&(if history_valid { 1.0f32 } else { 0.0f32 }).to_le_bytes());
        queue.write_buffer(&self.accum_params, 0, &params);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Orbital::VolumetricAccumulateBindGroup"),
            layout: &self.accumulate_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(current_froxels),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(history_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.froxel_sampler),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(output_view),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: self.accum_params.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Volumetric Accumulate"),
            ..Default::default()
        });
        pass.set_pipeline(&self.accumulate_pipeline);
        pass.set_bind_group(0, world_bind_group, &[]);
        pass.set_bind_group(1, &bind_group, &[]);
        pass.dispatch_workgroups(
            self.grid[0].div_ceil(4),
            self.grid[1].div_ceil(4),
            self.grid[2].div_ceil(4),
        );
        drop(pass);

        let output = output_view.clone();
        self.history_index = read_index;
        self.history_valid = true;
        self.previous_view = Some(current_view);
        self.previous_view_projection = Some(current_view_projection);
        self.previous_max_distance = self.current_max_distance;

        Some(output)
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

fn matrix_max_delta(a: &Matrix4<f32>, b: &Matrix4<f32>) -> f32 {
    let mut max_delta: f32 = 0.0;
    for (left, right) in [a.x, a.y, a.z, a.w].into_iter().zip([b.x, b.y, b.z, b.w]) {
        for (left, right) in [left.x, left.y, left.z, left.w]
            .into_iter()
            .zip([right.x, right.y, right.z, right.w])
        {
            max_delta = max_delta.max((left - right).abs());
        }
    }
    max_delta
}

fn matrix_to_bytes(matrix: &Matrix4<f32>) -> [u8; 64] {
    let mut bytes = [0u8; 64];
    let columns = [matrix.x, matrix.y, matrix.z, matrix.w];
    for (column_index, column) in columns.iter().enumerate() {
        for (row_index, value) in [column.x, column.y, column.z, column.w].iter().enumerate() {
            let offset = (column_index * 4 + row_index) * 4;
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}
