use std::sync::{Arc, RwLock};
use std::time::Instant;

use cgmath::{InnerSpace, Point3, Vector2};
use wgpu::{
    BindGroup, BindGroupEntry, Color, CommandEncoder, CommandEncoderDescriptor, CompareFunction,
    Device, FilterMode, IndexFormat, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, SamplerDescriptor, StoreOp,
    TextureFormat, TextureView, TextureViewDimension,
};

use orbital_camera::{Camera, CameraDescriptor};
use orbital_cull::CullResources;
use orbital_ibl_brdf::IblBrdf;
use orbital_material_shader::{MaterialShader, RenderQueue};
use orbital_model::Model;
use orbital_render_graph::{RenderGraph, ResourceId, TexturePool};
use orbital_shadow::{ShadowGpuData, ShadowRenderer};
use orbital_texture::Texture;
use orbital_world_environment::{GeneratedSkyParameters, WorldEnvironment};

use crate::deferred::DeferredLighting;
use crate::fxaa::Fxaa;
use crate::light_culling::LightCulling;
use crate::scene::RenderScene;
use crate::smaa::{SmaaQuality, SmaaTarget};
use crate::tonemap::Tonemap;
use crate::volumetric::VolumetricRenderer;

/// Anti-aliasing mode for the 3D scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AntiAliasing {
    Off,
    #[default]
    Fxaa,
    /// LUT-based SMAA 1x.
    Smaa,
}

/// Scene color format for the HDR intermediate target.
pub const SCENE_COLOR_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

/// G-buffer target formats: albedo+AO, normal+roughness, metallic+emissive.
pub const GBUFFER_COLOR_FORMATS: [TextureFormat; 3] = [
    TextureFormat::Rgba8Unorm,
    TextureFormat::Rgba16Float,
    TextureFormat::Rgba16Float,
];

/// Renderer passes scheduled by the frame graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ScenePass {
    Shadow,
    PrepareWorld,
    Sky,
    Gbuffer,
    LightCulling,
    DeferredLighting,
    ClearDepth,
    Models,
    Transparent,
    VolumetricInject,
    VolumetricComposite,
    Tonemap,
    Fxaa,
    Smaa,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelDraw {
    /// Opaque materials that cannot be deferred. Used in hybrid mode.
    ForwardOpaque,
    /// All opaque materials. Used when deferred rendering is disabled.
    ForwardOpaqueAll,
    Transparent,
}

fn bind_and_draw_model(
    pass: &mut wgpu::RenderPass<'_>,
    model: &Model,
    material: &MaterialShader,
    world_bind_group: &BindGroup,
    cull: Option<&CullResources>,
    model_index: usize,
) {
    pass.set_bind_group(0, world_bind_group, &[]);
    pass.set_bind_group(1, material.bind_group(), &[]);
    pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));

    if let Some(cr) = cull {
        let byte_off = cr.model_first_instance(model_index) as u64 * 64;
        pass.set_vertex_buffer(1, cr.filtered_instance_buffer().slice(byte_off..));
        pass.set_index_buffer(model.mesh().index_buffer().slice(..), IndexFormat::Uint32);
        pass.draw_indexed(
            0..model.mesh().index_count(),
            0,
            0..cr.visible_count(model_index),
        );
    } else {
        pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
        pass.set_index_buffer(model.mesh().index_buffer().slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..model.mesh().index_count(), 0, 0..model.instance_count());
    }
}

const RES_SHADOW_MAPS: ResourceId = ResourceId::new(1);
const RES_SCENE_COLOR: ResourceId = ResourceId::new(2);
const RES_DEPTH: ResourceId = ResourceId::new(3);
const RES_ENVIRONMENT: ResourceId = ResourceId::new(4);
const RES_WORLD: ResourceId = ResourceId::new(5);
const RES_SURFACE: ResourceId = ResourceId::new(6);
const RES_POST_LDR: ResourceId = ResourceId::new(7);
const RES_GBUFFER: ResourceId = ResourceId::new(8);
const RES_LIGHT_GRID: ResourceId = ResourceId::new(9);
const RES_VOLUMETRIC: ResourceId = ResourceId::new(10);
const RES_SCENE_COLOR_FINAL: ResourceId = ResourceId::new(11);

fn gbuffer_texture_descriptor(
    resolution: Vector2<u32>,
    format: TextureFormat,
    label: &'static str,
) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: resolution.x.max(1),
            height: resolution.y.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    }
}

fn add_scene_passes(
    graph: &mut RenderGraph<ScenePass>,
    anti_aliasing: AntiAliasing,
    has_deferred: bool,
    has_forward_opaque: bool,
    has_transparent: bool,
    has_volumetrics: bool,
) {
    graph.add_pass(ScenePass::Shadow, "shadow", &[], &[RES_SHADOW_MAPS]);
    graph.add_pass(
        ScenePass::PrepareWorld,
        "prepare-world",
        &[RES_SHADOW_MAPS],
        &[RES_WORLD],
    );
    graph.add_pass(
        ScenePass::Sky,
        "sky",
        &[RES_ENVIRONMENT, RES_WORLD],
        &[RES_SCENE_COLOR],
    );

    if has_deferred {
        graph.add_pass(
            ScenePass::Gbuffer,
            "gbuffer",
            &[RES_WORLD],
            &[RES_GBUFFER, RES_DEPTH],
        );
        graph.add_pass(
            ScenePass::LightCulling,
            "light-culling",
            &[RES_DEPTH, RES_WORLD],
            &[RES_LIGHT_GRID],
        );
        graph.add_pass(
            ScenePass::DeferredLighting,
            "deferred-lighting",
            &[
                RES_GBUFFER,
                RES_DEPTH,
                RES_SCENE_COLOR,
                RES_WORLD,
                RES_LIGHT_GRID,
            ],
            &[RES_SCENE_COLOR],
        );
    } else {
        graph.add_pass(ScenePass::ClearDepth, "clear-depth", &[], &[RES_DEPTH]);
    }

    if has_forward_opaque {
        graph.add_pass(
            ScenePass::Models,
            "models",
            &[RES_DEPTH, RES_SHADOW_MAPS, RES_ENVIRONMENT, RES_WORLD],
            &[RES_SCENE_COLOR, RES_DEPTH],
        );
    }

    if has_transparent {
        graph.add_pass(
            ScenePass::Transparent,
            "transparent",
            &[
                RES_SCENE_COLOR,
                RES_DEPTH,
                RES_SHADOW_MAPS,
                RES_ENVIRONMENT,
                RES_WORLD,
            ],
            &[RES_SCENE_COLOR],
        );
    }

    if has_volumetrics {
        graph.add_pass(
            ScenePass::VolumetricInject,
            "volumetric-inject",
            &[RES_WORLD],
            &[RES_VOLUMETRIC],
        );
        graph.add_pass(
            ScenePass::VolumetricComposite,
            "volumetric-composite",
            &[RES_SCENE_COLOR, RES_DEPTH, RES_VOLUMETRIC],
            &[RES_SCENE_COLOR_FINAL],
        );
    }

    let tonemap_source = if has_volumetrics {
        RES_SCENE_COLOR_FINAL
    } else {
        RES_SCENE_COLOR
    };
    let tonemap_target = match anti_aliasing {
        AntiAliasing::Off => RES_SURFACE,
        AntiAliasing::Fxaa | AntiAliasing::Smaa => RES_POST_LDR,
    };
    graph.add_pass(
        ScenePass::Tonemap,
        "tonemap",
        &[tonemap_source],
        &[tonemap_target],
    );
    match anti_aliasing {
        AntiAliasing::Off => {}
        AntiAliasing::Fxaa => {
            graph.add_pass(ScenePass::Fxaa, "fxaa", &[RES_POST_LDR], &[RES_SURFACE]);
        }
        AntiAliasing::Smaa => {
            graph.add_pass(ScenePass::Smaa, "smaa", &[RES_POST_LDR], &[RES_SURFACE]);
        }
    }
}

/// CPU time spent recording and submitting the scene.
///
/// The values are updated by [`Renderer::render`] and read by the runtime's
/// frame timing instrumentation.
#[derive(Debug, Clone, Copy, Default)]
pub struct RendererCpuTimings {
    /// Recording the shadow pass.
    pub shadow_ms: f64,
    /// Recording the skybox pass.
    pub skybox_ms: f64,
    /// Recording the model pass.
    pub models_ms: f64,
    /// Total command recording before submission.
    pub encode_ms: f64,
    /// `queue.submit` for the scene command buffer.
    pub submit_ms: f64,
    /// Timestamp resolve/copy/readback work.
    pub timestamp_readback_ms: f64,
}

/// Empty resources bound when a frame has no shadows.
///
/// The bind-group layout always includes the shadow slots, depth array, cube
/// array and comparison samplers, so they must point at valid GPU objects even
/// when shadow rendering is disabled.
struct ShadowFallbacks {
    slot_data_buffer: wgpu::Buffer,
    depth_view: TextureView,
    depth_sampler: wgpu::Sampler,
    cube_view: TextureView,
    cube_sampler: wgpu::Sampler,
    _depth_texture: wgpu::Texture,
    _cube_texture: wgpu::Texture,
}

impl ShadowFallbacks {
    fn new(device: &Device, queue: &Queue) -> Self {
        let slot_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Orbital::ShadowFallback::SlotData"),
            size: std::mem::size_of::<ShadowGpuData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&slot_data_buffer, 0, ShadowGpuData::new().as_bytes());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Orbital::ShadowFallback::Depth"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });

        let cube_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Orbital::ShadowFallback::CubeDepth"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let cube_view = cube_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(TextureViewDimension::CubeArray),
            ..Default::default()
        });

        let make_comparison_sampler = |label| {
            device.create_sampler(&SamplerDescriptor {
                label: Some(label),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                compare: Some(CompareFunction::LessEqual),
                ..Default::default()
            })
        };

        Self {
            slot_data_buffer,
            depth_view,
            depth_sampler: make_comparison_sampler("Orbital::ShadowFallback::Sampler"),
            cube_view,
            cube_sampler: make_comparison_sampler("Orbital::ShadowFallback::CubeSampler"),
            _depth_texture: depth_texture,
            _cube_texture: cube_texture,
        }
    }
}

/// Resources that keep the world bind group valid before the ECS has produced
/// a camera, light buffer or environment.
struct RendererFallbacks {
    camera: Camera,
    light_buffer: wgpu::Buffer,
    brdf: IblBrdf,
    env_diffuse: Texture,
    env_specular: Texture,
    sky_params: wgpu::Buffer,
    light_count: wgpu::Buffer,
    shadow: ShadowFallbacks,
    world_bind_group_layout: wgpu::BindGroupLayout,
}

impl RendererFallbacks {
    fn new(device: &Device, queue: &Queue) -> Self {
        let camera = Camera::from_descriptor(CameraDescriptor::default(), device, queue);
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Orbital::Fallback::LightBuffer"),
            size: 64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let brdf = IblBrdf::generate(device, queue);
        let env_diffuse = Texture::create_empty_cube_texture(
            Some("Orbital::Fallback::IBL Diffuse"),
            Vector2::new(1, 1),
            TextureFormat::R8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
            1,
            device,
        );
        let env_specular = Texture::create_empty_cube_texture(
            Some("Orbital::Fallback::IBL Specular"),
            Vector2::new(1, 1),
            TextureFormat::R8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
            1,
            device,
        );
        let sky_params = WorldEnvironment::make_sky_parameters_buffer(
            &GeneratedSkyParameters::default(),
            device,
        );
        let light_count = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Orbital::Fallback::LightCount"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow = ShadowFallbacks::new(device, queue);
        let world_bind_group_layout = orbital_material_shader::make_world_bind_group_layout(device);

        Self {
            camera,
            light_buffer,
            brdf,
            env_diffuse,
            env_specular,
            sky_params,
            light_count,
            shadow,
            world_bind_group_layout,
        }
    }
}

/// Identity of the resources referenced by the world bind group.
///
/// Buffer and texture contents can change every frame without invalidating the
/// binding; only replacing a resource (or reallocating shadow textures) does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorldBindGroupKey {
    camera: usize,
    light_buffer: usize,
    environment: usize,
    sky_params: usize,
    shadow_generation: Option<u64>,
}

struct CachedWorldBindGroup {
    key: WorldBindGroupKey,
    bind_group: BindGroup,
    // Keep the identities used by the key alive so allocator address reuse
    // cannot produce a false cache hit.
    _camera_source: Option<Arc<RwLock<Camera>>>,
    _light_buffer: Option<Arc<wgpu::Buffer>>,
    _environment: Option<Arc<WorldEnvironment>>,
}

pub struct Renderer {
    surface_texture_format: TextureFormat,
    resolution: Vector2<u32>,
    render_graph: RenderGraph<ScenePass>,
    texture_pool: TexturePool,
    frame_index: u64,
    anti_aliasing: AntiAliasing,
    smaa_quality: SmaaQuality,
    tonemap: Tonemap,
    fxaa: Fxaa,
    smaa: Option<SmaaTarget>,
    deferred_lighting: DeferredLighting,
    light_culling: LightCulling,
    volumetric: VolumetricRenderer,
    deferred_enabled: bool,
    shadow_renderer: Option<ShadowRenderer>,
    fallbacks: RendererFallbacks,
    world_bind_group_cache: Option<CachedWorldBindGroup>,
    timestamp_query_set: Option<wgpu::QuerySet>,
    timestamp_resolve_buffer: wgpu::Buffer,
    timestamp_staging_bufs: [wgpu::Buffer; 2],
    timestamp_read_frame: usize,
    prev_gpu_ns: [f64; 4],
    prev_cpu_timings: RendererCpuTimings,
}

impl Renderer {
    pub fn surface_texture_format(&self) -> &TextureFormat {
        &self.surface_texture_format
    }

    pub fn prev_gpu_ns(&self) -> [f64; 4] {
        self.prev_gpu_ns
    }

    pub fn prev_cpu_timings(&self) -> RendererCpuTimings {
        self.prev_cpu_timings
    }

    pub fn set_anti_aliasing(&mut self, anti_aliasing: AntiAliasing) {
        self.anti_aliasing = anti_aliasing;
        if anti_aliasing != AntiAliasing::Smaa {
            self.smaa = None;
        }
    }

    pub fn anti_aliasing(&self) -> AntiAliasing {
        self.anti_aliasing
    }

    pub fn set_smaa_quality(&mut self, quality: SmaaQuality) {
        if self.smaa_quality != quality {
            self.smaa_quality = quality;
            self.smaa = None;
        }
    }

    pub fn smaa_quality(&self) -> SmaaQuality {
        self.smaa_quality
    }

    /// Enable or disable the deferred path.
    ///
    /// Deferred rendering is the default for opaque PBR materials. Disabling
    /// it routes every opaque material through the forward path, which is
    /// useful for debugging and for scenes where a forward pass is a better
    /// fit.
    pub fn set_deferred_enabled(&mut self, enabled: bool) {
        self.deferred_enabled = enabled;
    }

    pub fn deferred_enabled(&self) -> bool {
        self.deferred_enabled
    }

    /// Human-readable frame-graph dump for debug overlays and logs.
    pub fn render_graph_debug_dump(&self) -> String {
        self.render_graph.debug_dump()
    }

    /// Graphviz representation of the frame graph.
    pub fn render_graph_dot(&self) -> String {
        self.render_graph.to_dot()
    }
}

const TS_COUNT: u32 = 4; // 0=shadow_start, 1=skybox_start, 2=models_start, 3=main_end
const TS_BUF_SIZE: u64 = TS_COUNT as u64 * 8; // 32 bytes

impl Renderer {
    pub fn new(
        surface_texture_format: TextureFormat,
        resolution: Vector2<u32>,
        device: &Device,
        queue: &Queue,
    ) -> Self {
        let has_timestamps = device
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

        let timestamp_query_set = if has_timestamps {
            Some(device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Orbital::TimestampQueries"),
                ty: wgpu::QueryType::Timestamp,
                count: TS_COUNT,
            }))
        } else {
            None
        };

        // Timestamp readback: resolve buffer (QUERY_RESOLVE | COPY_SRC) + double-buffered staging (COPY_DST | MAP_READ).
        // wgpu requires MAP_READ buffers to have ONLY MAP_READ | COPY_DST — no other flags.
        let timestamp_resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Orbital::TS_Resolve"),
            size: TS_BUF_SIZE,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let make_staging = |label| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: TS_BUF_SIZE,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };
        let timestamp_staging_bufs = [
            make_staging("Orbital::TS_Staging0"),
            make_staging("Orbital::TS_Staging1"),
        ];

        let fallbacks = RendererFallbacks::new(device, queue);
        let deferred_lighting = DeferredLighting::new(
            device,
            SCENE_COLOR_FORMAT,
            &fallbacks.world_bind_group_layout,
        );
        let light_culling = LightCulling::new(
            device,
            queue,
            resolution,
            &fallbacks.world_bind_group_layout,
        );
        let volumetric = VolumetricRenderer::new(
            device,
            queue,
            SCENE_COLOR_FORMAT,
            &fallbacks.world_bind_group_layout,
        );

        Self {
            surface_texture_format,
            resolution,
            render_graph: RenderGraph::default(),
            texture_pool: TexturePool::new(3),
            frame_index: 0,
            anti_aliasing: AntiAliasing::default(),
            smaa_quality: SmaaQuality::default(),
            tonemap: Tonemap::new(device, surface_texture_format),
            fxaa: Fxaa::new(device, queue, surface_texture_format, resolution),
            smaa: None,
            deferred_lighting,
            light_culling,
            volumetric,
            deferred_enabled: true,
            shadow_renderer: None,
            fallbacks,
            world_bind_group_cache: None,
            timestamp_query_set,
            timestamp_resolve_buffer,
            timestamp_staging_bufs,
            timestamp_read_frame: 0,
            prev_gpu_ns: [0.0; 4],
            prev_cpu_timings: RendererCpuTimings::default(),
        }
    }

    pub fn enable_shadows(
        &mut self,
        device: &Device,
        queue: &Queue,
        max_slots: u32,
        resolution: u32,
    ) {
        self.shadow_renderer = Some(ShadowRenderer::new(device, queue, max_slots, resolution));
        self.world_bind_group_cache = None;
    }

    pub fn shadow_renderer(&self) -> Option<&ShadowRenderer> {
        self.shadow_renderer.as_ref()
    }

    pub fn shadow_renderer_mut(&mut self) -> Option<&mut ShadowRenderer> {
        self.shadow_renderer.as_mut()
    }

    pub fn set_surface_texture_format(
        &mut self,
        surface_texture_format: TextureFormat,
        device: &Device,
        queue: &Queue,
    ) {
        self.surface_texture_format = surface_texture_format;
        self.tonemap = Tonemap::new(device, surface_texture_format);
        self.fxaa = Fxaa::new(device, queue, surface_texture_format, self.resolution);
        self.smaa = None;
    }

    pub fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue) {
        self.resolution = resolution;
        self.fxaa.set_resolution(queue, resolution);
        if let Some(smaa) = &mut self.smaa {
            smaa.resize(device, resolution.x.max(1), resolution.y.max(1));
        }
        self.light_culling.ensure(device, queue, resolution);
    }

    /// Returns the camera uniform buffer for this frame.
    ///
    /// Falls back to the renderer's default camera when no ECS camera has been
    /// realized yet, so overlays and the scene always have a valid binding.
    pub fn camera_buffer<'a>(&'a self, scene: &'a RenderScene) -> &'a wgpu::Buffer {
        scene
            .camera
            .as_ref()
            .map(|camera| camera.buffer())
            .unwrap_or_else(|| self.fallbacks.camera.camera_buffer())
    }

    fn world_bind_group_key(&self, scene: &RenderScene) -> WorldBindGroupKey {
        let camera = scene
            .camera
            .as_ref()
            .map(|camera| camera.source_id())
            .unwrap_or_else(|| {
                self.fallbacks.camera.camera_buffer() as *const wgpu::Buffer as usize
            });
        let light_buffer = scene
            .light_buffer
            .as_ref()
            .map(|buffer| Arc::as_ptr(buffer) as usize)
            .unwrap_or_else(|| &self.fallbacks.light_buffer as *const wgpu::Buffer as usize);
        let environment = scene
            .environment
            .as_ref()
            .map(|environment| Arc::as_ptr(environment) as usize)
            .unwrap_or(0);
        let sky_params = scene
            .environment
            .as_ref()
            .and_then(|environment| environment.sky_parameters_buffer())
            .map(|buffer| buffer as *const wgpu::Buffer as usize)
            .unwrap_or_else(|| &self.fallbacks.sky_params as *const wgpu::Buffer as usize);
        let shadow_generation = self
            .shadow_renderer
            .as_ref()
            .map(|shadow_renderer| shadow_renderer.generation());

        WorldBindGroupKey {
            camera,
            light_buffer,
            environment,
            sky_params,
            shadow_generation,
        }
    }

    fn world_bind_group(
        &mut self,
        scene: &RenderScene,
        device: &Device,
        queue: &Queue,
    ) -> BindGroup {
        // The light count changes independently of the light buffer identity.
        let light_count_bytes: [u8; 16] = {
            let mut data = [0u8; 16];
            data[..4].copy_from_slice(&scene.light_count.to_le_bytes());
            data
        };
        queue.write_buffer(&self.fallbacks.light_count, 0, &light_count_bytes);

        let key = self.world_bind_group_key(scene);
        if let Some(cached) = &self.world_bind_group_cache
            && cached.key == key
        {
            return cached.bind_group.clone();
        }

        let bind_group = self.create_world_bind_group(scene, device);
        self.world_bind_group_cache = Some(CachedWorldBindGroup {
            key,
            bind_group: bind_group.clone(),
            _camera_source: scene.camera.as_ref().map(|camera| camera.source()),
            _light_buffer: scene.light_buffer.clone(),
            _environment: scene.environment.clone(),
        });
        bind_group
    }

    fn create_world_bind_group(&self, scene: &RenderScene, device: &Device) -> BindGroup {
        let camera_buffer = self.camera_buffer(scene);
        let light_buffer = scene
            .light_buffer
            .as_deref()
            .unwrap_or(&self.fallbacks.light_buffer);

        let (
            diffuse_env_view,
            diffuse_env_sampler,
            specular_env_view,
            specular_env_sampler,
            sky_params,
        ) = match &scene.environment {
            Some(environment) => (
                environment.ibl_diffuse().view(),
                environment.ibl_diffuse().sampler(),
                environment.ibl_specular().view(),
                environment.ibl_specular().sampler(),
                environment
                    .sky_parameters_buffer()
                    .unwrap_or(&self.fallbacks.sky_params),
            ),
            None => (
                self.fallbacks.env_diffuse.view(),
                self.fallbacks.env_diffuse.sampler(),
                self.fallbacks.env_specular.view(),
                self.fallbacks.env_specular.sampler(),
                &self.fallbacks.sky_params,
            ),
        };

        let brdf_texture = self.fallbacks.brdf.texture_ref();

        let (
            shadow_slot_buffer,
            shadow_depth_view,
            shadow_sampler,
            cube_shadow_view,
            cube_shadow_sampler,
        ) = match &self.shadow_renderer {
            Some(shadow_renderer) => (
                shadow_renderer.slot_data_buffer(),
                shadow_renderer.depth_texture().view(),
                shadow_renderer.sampler(),
                shadow_renderer.cube_depth_texture().view(),
                shadow_renderer.cube_sampler(),
            ),
            None => (
                &self.fallbacks.shadow.slot_data_buffer,
                &self.fallbacks.shadow.depth_view,
                &self.fallbacks.shadow.depth_sampler,
                &self.fallbacks.shadow.cube_view,
                &self.fallbacks.shadow.cube_sampler,
            ),
        };

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("World Bind Group"),
            layout: &self.fallbacks.world_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        camera_buffer.as_entire_buffer_binding(),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(
                        light_buffer.as_entire_buffer_binding(),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(diffuse_env_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(diffuse_env_sampler),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(specular_env_view),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(specular_env_sampler),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(brdf_texture.view()),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(brdf_texture.sampler()),
                },
                BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Buffer(
                        shadow_slot_buffer.as_entire_buffer_binding(),
                    ),
                },
                BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(shadow_depth_view),
                },
                BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::Sampler(shadow_sampler),
                },
                BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::TextureView(cube_shadow_view),
                },
                BindGroupEntry {
                    binding: 12,
                    resource: wgpu::BindingResource::Sampler(cube_shadow_sampler),
                },
                BindGroupEntry {
                    binding: 13,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.fallbacks.light_count,
                        offset: 0,
                        size: None,
                    }),
                },
                BindGroupEntry {
                    binding: 14,
                    resource: wgpu::BindingResource::Buffer(sky_params.as_entire_buffer_binding()),
                },
            ],
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        target_view: &TextureView,
        scene: &RenderScene,
        device: &Device,
        queue: &Queue,
        cull: Option<&CullResources>,
        dirty_set: &std::collections::HashSet<u32>,
        overlay: Option<&mut dyn FnMut(&mut wgpu::CommandEncoder)>,
    ) {
        let ms = |duration: std::time::Duration| duration.as_secs_f64() * 1000.0;
        let render_start = Instant::now();

        self.frame_index = self.frame_index.wrapping_add(1);
        self.texture_pool.begin_frame(self.frame_index);

        // SMAA owns the LDR color target. Create it lazily so Fxaa/Off users do
        // not pay for the extra targets.
        if self.anti_aliasing == AntiAliasing::Smaa && self.smaa.is_none() {
            self.smaa = Some(SmaaTarget::new(
                device,
                queue,
                self.resolution.x.max(1),
                self.resolution.y.max(1),
                self.surface_texture_format,
                self.smaa_quality,
            ));
        }
        let smaa = self.smaa.take();
        let smaa_view = smaa.as_ref().map(|smaa| smaa.color_target().clone());

        let depth_descriptor = wgpu::TextureDescriptor {
            label: Some("Orbital::Depth"),
            size: wgpu::Extent3d {
                width: self.resolution.x.max(1),
                height: self.resolution.y.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let depth = self.texture_pool.acquire(device, &depth_descriptor);

        let scene_color_descriptor = wgpu::TextureDescriptor {
            label: Some("Orbital::SceneColor"),
            size: wgpu::Extent3d {
                width: self.resolution.x.max(1),
                height: self.resolution.y.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SCENE_COLOR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let scene_color = self.texture_pool.acquire(device, &scene_color_descriptor);

        let post_ldr = (self.anti_aliasing == AntiAliasing::Fxaa).then(|| {
            self.texture_pool.acquire(
                device,
                &wgpu::TextureDescriptor {
                    label: Some("Orbital::PostLdr"),
                    size: wgpu::Extent3d {
                        width: self.resolution.x.max(1),
                        height: self.resolution.y.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.surface_texture_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
            )
        });

        let mut has_deferred = false;
        let mut has_forward_opaque = false;
        let mut has_transparent = false;
        for model in &scene.models {
            for material in model.materials() {
                match material.render_queue() {
                    RenderQueue::Opaque => {
                        if self.deferred_enabled && material.gbuffer_pipeline().is_some() {
                            has_deferred = true;
                        } else {
                            has_forward_opaque = true;
                        }
                    }
                    RenderQueue::Transparent => has_transparent = true,
                }
            }
        }

        if !has_deferred {
            self.deferred_lighting.clear_cache();
            self.light_culling.clear_cache();
        }

        let shadow_models: Vec<Arc<Model>> = scene
            .models
            .iter()
            .filter(|model| model.casts_shadows())
            .cloned()
            .collect();

        let gbuffer = if has_deferred {
            Some([
                self.texture_pool.acquire(
                    device,
                    &gbuffer_texture_descriptor(
                        self.resolution,
                        GBUFFER_COLOR_FORMATS[0],
                        "Orbital::Gbuffer0",
                    ),
                ),
                self.texture_pool.acquire(
                    device,
                    &gbuffer_texture_descriptor(
                        self.resolution,
                        GBUFFER_COLOR_FORMATS[1],
                        "Orbital::Gbuffer1",
                    ),
                ),
                self.texture_pool.acquire(
                    device,
                    &gbuffer_texture_descriptor(
                        self.resolution,
                        GBUFFER_COLOR_FORMATS[2],
                        "Orbital::Gbuffer2",
                    ),
                ),
            ])
        } else {
            None
        };

        if has_deferred {
            self.light_culling.ensure(device, queue, self.resolution);
        }

        let volumetrics = scene.volumetrics.filter(|settings| settings.enabled);
        let has_volumetrics = volumetrics.is_some();
        if !has_volumetrics {
            self.volumetric.reset_history();
        }
        let volumetric_targets = if let Some(settings) = volumetrics {
            self.volumetric.prepare(
                queue,
                &settings,
                self.resolution,
                self.frame_index,
                scene.camera.as_ref().map(|camera| camera.view_matrix),
                scene
                    .camera
                    .as_ref()
                    .map(|camera| camera.perspective_view_projection),
            );
            let grid = self.volumetric.grid();
            let froxel = self.texture_pool.acquire(
                device,
                &wgpu::TextureDescriptor {
                    label: Some("Orbital::VolumetricFroxels"),
                    size: wgpu::Extent3d {
                        width: grid[0],
                        height: grid[1],
                        depth_or_array_layers: grid[2],
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D3,
                    format: SCENE_COLOR_FORMAT,
                    usage: wgpu::TextureUsages::STORAGE_BINDING
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
            );
            let final_color = self.texture_pool.acquire(
                device,
                &gbuffer_texture_descriptor(
                    self.resolution,
                    SCENE_COLOR_FORMAT,
                    "Orbital::VolumetricSceneColor",
                ),
            );
            Some((froxel, final_color))
        } else {
            None
        };

        let mut volumetric_accumulated: Option<TextureView> = None;
        let mut graph = std::mem::take(&mut self.render_graph);
        graph.begin_frame();
        add_scene_passes(
            &mut graph,
            self.anti_aliasing,
            has_deferred,
            has_forward_opaque,
            has_transparent,
            has_volumetrics,
        );
        graph.compile().expect("render graph must be acyclic");

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Orbital::Render::Encoder"),
        });

        // Write timestamp 0: start of shadow pass
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 0);
        }

        let mut world_bind_group: Option<BindGroup> = None;
        let camera_position = scene.camera.as_ref().map(|camera| camera.position);
        let mut shadow_ms = 0.0;
        let mut skybox_ms = 0.0;
        let mut models_ms = 0.0;

        graph
            .execute(
                &mut command_encoder,
                device,
                queue,
                |pass, context| match pass {
                    ScenePass::Shadow => {
                        if let Some(shadow_renderer) = self.shadow_renderer.as_mut()
                            && let Some(camera) = &scene.camera
                        {
                            shadow_renderer.render(
                                context.encoder,
                                &shadow_models,
                                &scene.shadow_lights,
                                &camera.perspective_view_projection,
                                camera.near,
                                camera.far,
                                context.device,
                                context.queue,
                                dirty_set,
                            );
                        }
                        if let Some(qs) = &self.timestamp_query_set {
                            context.encoder.write_timestamp(qs, 1);
                        }
                    }
                    ScenePass::PrepareWorld => {
                        world_bind_group = Some(self.world_bind_group(scene, device, queue));
                    }
                    ScenePass::Sky => {
                        self.render_sky_box(
                            &scene_color.view,
                            scene
                                .environment
                                .as_deref()
                                .map(|environment| environment.material_shader()),
                            world_bind_group.as_ref(),
                            context.encoder,
                        );
                        if let Some(qs) = &self.timestamp_query_set {
                            context.encoder.write_timestamp(qs, 2);
                        }
                    }
                    ScenePass::Gbuffer => {
                        if let (Some(bind_group), Some(gbuffer)) =
                            (world_bind_group.as_ref(), gbuffer.as_ref())
                        {
                            self.render_gbuffer_models(
                                &scene.models,
                                gbuffer,
                                bind_group,
                                context.encoder,
                                cull,
                                &depth.view,
                            );
                        }
                    }
                    ScenePass::LightCulling => {
                        if let Some(bind_group) = world_bind_group.as_ref() {
                            self.light_culling.record(
                                context.device,
                                context.encoder,
                                bind_group,
                                &depth.view,
                            );
                        }
                    }
                    ScenePass::DeferredLighting => {
                        if let (Some(bind_group), Some(gbuffer)) =
                            (world_bind_group.as_ref(), gbuffer.as_ref())
                        {
                            let tile_counts = self.light_culling.counts().clone();
                            let tile_indices = self.light_culling.indices().clone();
                            let tile_params = self.light_culling.params().clone();
                            self.deferred_lighting.record(
                                context.device,
                                context.encoder,
                                bind_group,
                                &gbuffer[0].view,
                                &gbuffer[1].view,
                                &gbuffer[2].view,
                                &depth.view,
                                &tile_counts,
                                &tile_indices,
                                &tile_params,
                                &scene_color.view,
                            );
                        }
                    }
                    ScenePass::ClearDepth => {
                        self.clear_depth(context.encoder, &depth.view);
                    }
                    ScenePass::Models => {
                        if let Some(bind_group) = world_bind_group.as_ref() {
                            let draw = if has_deferred {
                                ModelDraw::ForwardOpaque
                            } else {
                                ModelDraw::ForwardOpaqueAll
                            };
                            self.render_models(
                                &scene.models,
                                &scene_color.view,
                                bind_group,
                                context.encoder,
                                cull,
                                &depth.view,
                                draw,
                                camera_position,
                            );
                        }
                    }
                    ScenePass::Transparent => {
                        if let Some(bind_group) = world_bind_group.as_ref() {
                            self.render_models(
                                &scene.models,
                                &scene_color.view,
                                bind_group,
                                context.encoder,
                                cull,
                                &depth.view,
                                ModelDraw::Transparent,
                                camera_position,
                            );
                        }
                    }
                    ScenePass::VolumetricInject => {
                        if let (Some(bind_group), Some(targets)) =
                            (world_bind_group.as_ref(), volumetric_targets.as_ref())
                        {
                            self.volumetric.record_inject(
                                context.device,
                                context.encoder,
                                bind_group,
                                &targets.0.view,
                            );
                            volumetric_accumulated = self.volumetric.record_accumulate(
                                context.device,
                                context.queue,
                                context.encoder,
                                bind_group,
                                &targets.0.view,
                            );
                        }
                    }
                    ScenePass::VolumetricComposite => {
                        if let (Some(bind_group), Some(targets)) =
                            (world_bind_group.as_ref(), volumetric_targets.as_ref())
                        {
                            let froxels =
                                volumetric_accumulated.as_ref().unwrap_or(&targets.0.view);
                            self.volumetric.record_composite(
                                context.device,
                                context.encoder,
                                bind_group,
                                &scene_color.view,
                                &depth.view,
                                froxels,
                                &targets.1.view,
                            );
                        }
                    }
                    ScenePass::Tonemap => {
                        if let Some(qs) = &self.timestamp_query_set {
                            context.encoder.write_timestamp(qs, 3);
                        }
                        let tonemap_target = match self.anti_aliasing {
                            AntiAliasing::Off => target_view,
                            AntiAliasing::Fxaa => post_ldr
                                .as_ref()
                                .map(|post_ldr| &post_ldr.view)
                                .unwrap_or(target_view),
                            AntiAliasing::Smaa => smaa_view.as_ref().expect("SMAA target view"),
                        };
                        let tonemap_source = volumetric_targets
                            .as_ref()
                            .map(|targets| &targets.1.view)
                            .unwrap_or(&scene_color.view);
                        self.tonemap.record(
                            context.device,
                            context.encoder,
                            tonemap_source,
                            tonemap_target,
                        );
                    }
                    ScenePass::Fxaa => {
                        if let Some(post_ldr) = &post_ldr {
                            self.fxaa.record(
                                context.device,
                                context.encoder,
                                &post_ldr.view,
                                target_view,
                            );
                        }
                    }
                    ScenePass::Smaa => {
                        if let Some(smaa) = smaa.as_ref() {
                            smaa.record(context.encoder, target_view);
                        }
                    }
                },
            )
            .expect("render graph must be compiled before execution");
        self.smaa = smaa;

        if let Some(overlay) = overlay {
            overlay(&mut command_encoder);
        }

        for timing in graph.timings() {
            match timing.pass {
                ScenePass::Shadow => shadow_ms = ms(timing.cpu),
                ScenePass::Sky => skybox_ms = ms(timing.cpu),
                ScenePass::Gbuffer
                | ScenePass::LightCulling
                | ScenePass::DeferredLighting
                | ScenePass::Models
                | ScenePass::VolumetricInject
                | ScenePass::VolumetricComposite => {
                    models_ms += ms(timing.cpu);
                }
                ScenePass::Transparent => models_ms += ms(timing.cpu),
                ScenePass::PrepareWorld => skybox_ms += ms(timing.cpu),
                ScenePass::ClearDepth | ScenePass::Tonemap | ScenePass::Fxaa | ScenePass::Smaa => {}
            }
        }
        self.render_graph = graph;

        // Resolve this frame's timestamps into the current staging buffer in the
        // same command encoder as the scene. This keeps the frame to one submit.
        if let Some(timestamp_query_set) = &self.timestamp_query_set {
            let current = self.timestamp_read_frame & 1;
            command_encoder.resolve_query_set(
                timestamp_query_set,
                0..TS_COUNT,
                &self.timestamp_resolve_buffer,
                0,
            );
            command_encoder.copy_buffer_to_buffer(
                &self.timestamp_resolve_buffer,
                0,
                &self.timestamp_staging_bufs[current],
                0,
                TS_BUF_SIZE,
            );
        }

        let submit_start = Instant::now();
        queue.submit(vec![command_encoder.finish()]);
        let submit_ms = ms(submit_start.elapsed());
        let timestamp_readback_start = Instant::now();
        self.texture_pool.end_frame();

        // Read the previous frame's staging buffer. It was submitted at least one
        // frame ago, so a non-blocking poll usually has the data ready without
        // stalling the CPU/GPU overlap.
        if self.timestamp_query_set.is_some() {
            let current = self.timestamp_read_frame & 1;
            let previous = 1 - current;

            if self.timestamp_read_frame > 0 {
                let previous_buffer = &self.timestamp_staging_bufs[previous];
                let previous_slice = previous_buffer.slice(..);
                let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let done2 = done.clone();
                previous_slice.map_async(wgpu::MapMode::Read, move |_| {
                    done2.store(true, std::sync::atomic::Ordering::Relaxed);
                });
                let _ = device.poll(wgpu::PollType::Poll);

                if done.load(std::sync::atomic::Ordering::Relaxed)
                    && let Ok(data) = previous_slice.get_mapped_range()
                {
                    let mut ns = [0.0f64; TS_COUNT as usize];
                    for (index, value) in ns.iter_mut().enumerate() {
                        let start = index * 8;
                        let bytes: [u8; 8] = data[start..start + 8].try_into().unwrap();
                        *value = u64::from_ne_bytes(bytes) as f64;
                    }
                    self.prev_gpu_ns = ns;
                }
                previous_buffer.unmap();
            }

            self.timestamp_read_frame += 1;
        }

        self.prev_cpu_timings = RendererCpuTimings {
            shadow_ms,
            skybox_ms,
            models_ms,
            encode_ms: ms(submit_start - render_start),
            submit_ms,
            timestamp_readback_ms: ms(timestamp_readback_start.elapsed()),
        };
    }

    fn render_sky_box(
        &self,
        target_view: &TextureView,
        sky_box_shader: Option<&MaterialShader>,
        world_bind_group: Option<&BindGroup>,
        command_encoder: &mut CommandEncoder,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("RenderPass::SkyBox"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let (Some(shader), Some(bind_group)) = (sky_box_shader, world_bind_group) {
            render_pass.set_pipeline(shader.pipeline());
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_models(
        &self,
        models: &[Arc<Model>],
        target_view: &TextureView,
        world_bind_group: &BindGroup,
        command_encoder: &mut CommandEncoder,
        cull: Option<&CullResources>,
        depth_view: &TextureView,
        draw: ModelDraw,
        camera_position: Option<Point3<f32>>,
    ) {
        let mut draws: Vec<(usize, usize, f32)> = Vec::new();

        for (model_index, model) in models.iter().enumerate() {
            let distance = if draw == ModelDraw::Transparent {
                match (model.instance_center(), camera_position) {
                    (Some(center), Some(camera)) => (center - camera).magnitude2(),
                    _ => 0.0,
                }
            } else {
                0.0
            };

            for (material_index, material) in model.materials().iter().enumerate() {
                let include = match draw {
                    ModelDraw::ForwardOpaque => {
                        material.render_queue() == RenderQueue::Opaque
                            && material.gbuffer_pipeline().is_none()
                    }
                    ModelDraw::ForwardOpaqueAll => material.render_queue() == RenderQueue::Opaque,
                    ModelDraw::Transparent => material.render_queue() == RenderQueue::Transparent,
                };
                if include {
                    draws.push((model_index, material_index, distance));
                }
            }
        }

        if draws.is_empty() {
            return;
        }

        if draw == ModelDraw::Transparent {
            // Back-to-front for alpha blending.
            draws.sort_by(|a, b| b.2.total_cmp(&a.2));
        }

        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some(match draw {
                ModelDraw::ForwardOpaque | ModelDraw::ForwardOpaqueAll => {
                    "Forward Model RenderPass"
                }
                ModelDraw::Transparent => "Transparent RenderPass",
            }),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        for (model_index, material_index, _) in draws {
            let model = &models[model_index];
            let material = &model.materials()[material_index];

            render_pass.set_pipeline(material.pipeline());
            bind_and_draw_model(
                &mut render_pass,
                model,
                material,
                world_bind_group,
                cull,
                model_index,
            );
        }
    }

    fn render_gbuffer_models(
        &self,
        models: &[Arc<Model>],
        gbuffer: &[orbital_render_graph::PooledTexture; 3],
        world_bind_group: &BindGroup,
        command_encoder: &mut CommandEncoder,
        cull: Option<&CullResources>,
        depth_view: &TextureView,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("G-buffer RenderPass"),
            color_attachments: &[
                Some(RenderPassColorAttachment {
                    view: &gbuffer[0].view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                }),
                Some(RenderPassColorAttachment {
                    view: &gbuffer[1].view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                }),
                Some(RenderPassColorAttachment {
                    view: &gbuffer[2].view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                }),
            ],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        for (model_index, model) in models.iter().enumerate() {
            for material in model.materials() {
                if material.render_queue() != RenderQueue::Opaque {
                    continue;
                }
                let Some(pipeline) = material.gbuffer_pipeline() else {
                    continue;
                };

                render_pass.set_pipeline(pipeline);
                bind_and_draw_model(
                    &mut render_pass,
                    model,
                    material,
                    world_bind_group,
                    cull,
                    model_index,
                );
            }
        }
    }

    fn clear_depth(&self, command_encoder: &mut CommandEncoder, depth_view: &TextureView) {
        let _ = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("ClearDepth RenderPass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    /// Renders 2D geometry (solid-color shapes) on top of the existing scene.
    ///
    /// Call this after `render()` to add 2D overlays.
    /// The 2D content is rendered with alpha blending and no depth testing.
    pub fn render_2d(
        &self,
        renderer_2d: &crate::Renderer2D,
        bind_group: &BindGroup,
        vertices: &[orbital_2d::Vertex2D],
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
    ) {
        if vertices.is_empty() {
            return;
        }

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Orbital::Render2D::Encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("RenderPass::2D"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Upload vertex data
            let byte_data = unsafe {
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    std::mem::size_of_val(vertices),
                )
            };
            queue.write_buffer(renderer_2d.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(renderer_2d.pipeline());
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, renderer_2d.vertex_buffer().slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        queue.submit(std::iter::once(command_encoder.finish()));
    }

    /// Renders UI backgrounds (SDF rounded rectangles) on top of the existing scene.
    ///
    /// Call this after `render()` to add UI elements.
    /// The UI content is rendered with alpha blending and no depth testing.
    pub fn render_ui(
        &self,
        ui_renderer: &crate::UiRenderer,
        vertices: &[orbital_2d::Vertex2D],
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
    ) {
        if vertices.is_empty() {
            return;
        }

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Orbital::RenderUI::Encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("RenderPass::UI"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Upload vertex data
            let byte_data = unsafe {
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    std::mem::size_of_val(vertices),
                )
            };
            queue.write_buffer(ui_renderer.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(ui_renderer.pipeline());
            render_pass.set_vertex_buffer(0, ui_renderer.vertex_buffer().slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        queue.submit(std::iter::once(command_encoder.finish()));
    }

    /// Renders text (SDF) on top of the existing scene.
    ///
    /// Call this after `render()` to add text labels.
    /// The text is rendered with alpha blending and no depth testing.
    pub fn render_text(
        &self,
        text_renderer: &crate::TextRenderer,
        atlas_bind_group: &BindGroup,
        vertices: &[orbital_2d::Vertex2D],
        target_view: &TextureView,
        device: &Device,
        queue: &Queue,
    ) {
        if vertices.is_empty() || !text_renderer.has_atlas() {
            return;
        }

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Orbital::RenderText::Encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("RenderPass::Text"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Upload vertex data
            let byte_data = unsafe {
                std::slice::from_raw_parts(
                    vertices.as_ptr() as *const u8,
                    std::mem::size_of_val(vertices),
                )
            };
            queue.write_buffer(text_renderer.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(text_renderer.pipeline());
            render_pass.set_bind_group(1, atlas_bind_group, &[]);
            render_pass.set_vertex_buffer(0, text_renderer.vertex_buffer().slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        queue.submit(std::iter::once(command_encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_order(
        anti_aliasing: AntiAliasing,
        has_deferred: bool,
        has_forward_opaque: bool,
        has_transparent: bool,
        has_volumetrics: bool,
    ) -> Vec<&'static str> {
        let mut graph = RenderGraph::new();
        add_scene_passes(
            &mut graph,
            anti_aliasing,
            has_deferred,
            has_forward_opaque,
            has_transparent,
            has_volumetrics,
        );
        graph.compile().unwrap();
        graph
            .ordered_passes()
            .map(|(_, name)| name)
            .collect::<Vec<_>>()
    }

    #[test]
    fn scene_graph_with_forward_opaque_clears_depth_first() {
        assert_eq!(
            graph_order(AntiAliasing::Off, false, true, false, false),
            [
                "shadow",
                "prepare-world",
                "sky",
                "clear-depth",
                "models",
                "tonemap"
            ]
        );
    }

    #[test]
    fn scene_graph_with_deferred_opaque_skips_forward_models() {
        assert_eq!(
            graph_order(AntiAliasing::Off, true, false, false, false),
            [
                "shadow",
                "prepare-world",
                "sky",
                "gbuffer",
                "light-culling",
                "deferred-lighting",
                "tonemap"
            ]
        );
    }

    #[test]
    fn scene_graph_with_fxaa_adds_post_pass() {
        assert_eq!(
            graph_order(AntiAliasing::Fxaa, true, false, false, false),
            [
                "shadow",
                "prepare-world",
                "sky",
                "gbuffer",
                "light-culling",
                "deferred-lighting",
                "tonemap",
                "fxaa"
            ]
        );
    }

    #[test]
    fn scene_graph_with_smaa_adds_post_pass() {
        assert_eq!(
            graph_order(AntiAliasing::Smaa, true, false, false, false),
            [
                "shadow",
                "prepare-world",
                "sky",
                "gbuffer",
                "light-culling",
                "deferred-lighting",
                "tonemap",
                "smaa"
            ]
        );
    }

    #[test]
    fn scene_graph_with_smaa_and_volumetrics_orders_post_chain() {
        assert_eq!(
            graph_order(AntiAliasing::Smaa, true, false, true, true),
            [
                "shadow",
                "prepare-world",
                "sky",
                "gbuffer",
                "light-culling",
                "deferred-lighting",
                "transparent",
                "volumetric-inject",
                "volumetric-composite",
                "tonemap",
                "smaa"
            ]
        );
    }

    #[test]
    fn scene_graph_hybrid_runs_deferred_then_forward_and_transparent() {
        assert_eq!(
            graph_order(AntiAliasing::Off, true, true, true, false),
            [
                "shadow",
                "prepare-world",
                "sky",
                "gbuffer",
                "light-culling",
                "deferred-lighting",
                "models",
                "transparent",
                "tonemap"
            ]
        );
    }

    #[test]
    fn scene_graph_with_only_transparency_clears_depth_first() {
        assert_eq!(
            graph_order(AntiAliasing::Off, false, false, true, false),
            [
                "shadow",
                "prepare-world",
                "sky",
                "clear-depth",
                "transparent",
                "tonemap"
            ]
        );
    }

    #[test]
    fn scene_graph_runs_volumetrics_before_tonemap() {
        assert_eq!(
            graph_order(AntiAliasing::Off, true, false, true, true),
            [
                "shadow",
                "prepare-world",
                "sky",
                "gbuffer",
                "light-culling",
                "deferred-lighting",
                "transparent",
                "volumetric-inject",
                "volumetric-composite",
                "tonemap"
            ]
        );
    }

    #[test]
    fn renderer_creates_pipelines_on_noop_backend() {
        let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
        let mut renderer = Renderer::new(
            TextureFormat::Bgra8UnormSrgb,
            Vector2::new(320, 180),
            &device,
            &queue,
        );
        renderer.enable_shadows(&device, &queue, 4, 256);
    }

    #[test]
    fn renderer_renders_smaa_and_volumetrics_on_noop_backend() {
        let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
        let mut renderer = Renderer::new(
            TextureFormat::Bgra8UnormSrgb,
            Vector2::new(320, 180),
            &device,
            &queue,
        );
        renderer.set_anti_aliasing(AntiAliasing::Smaa);
        let target = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Renderer noop target"),
                size: wgpu::Extent3d {
                    width: 320,
                    height: 180,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default());

        let mut world = orbital_ecs::World::new();
        world.insert_resource(orbital_ecs_bridge::VolumetricSettings {
            enabled: true,
            quality: orbital_ecs_bridge::VolumetricQuality::Low,
            ..Default::default()
        });
        let scene = RenderScene::from_world(&world);
        renderer.render(
            &target,
            &scene,
            &device,
            &queue,
            None,
            &std::collections::HashSet::new(),
            None,
        );
    }

    #[test]
    fn renderer_renders_volumetrics_on_noop_backend() {
        let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
        let mut renderer = Renderer::new(
            TextureFormat::Bgra8UnormSrgb,
            Vector2::new(320, 180),
            &device,
            &queue,
        );
        let target = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Renderer noop target"),
                size: wgpu::Extent3d {
                    width: 320,
                    height: 180,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default());

        let mut world = orbital_ecs::World::new();
        world.insert_resource(orbital_ecs_bridge::VolumetricSettings {
            enabled: true,
            ..Default::default()
        });
        let scene = RenderScene::from_world(&world);
        renderer.render(
            &target,
            &scene,
            &device,
            &queue,
            None,
            &std::collections::HashSet::new(),
            None,
        );
    }

    #[test]
    fn renderer_accumulates_volumetric_history_on_noop_backend() {
        use cgmath::Quaternion;
        use orbital_ecs_bridge::{ActiveCamera, CameraStore};

        let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
        let mut renderer = Renderer::new(
            TextureFormat::Bgra8UnormSrgb,
            Vector2::new(320, 180),
            &device,
            &queue,
        );
        let target = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Renderer temporal noop target"),
                size: wgpu::Extent3d {
                    width: 320,
                    height: 180,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default());

        let mut world = orbital_ecs::World::new();
        world.insert_resource(orbital_ecs_bridge::VolumetricSettings {
            enabled: true,
            ..Default::default()
        });
        let camera = Camera::new(
            Point3::new(0.0, 1.0, 5.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
            45.0,
            16.0 / 9.0,
            0.1,
            100.0,
            2.2,
            &device,
            &queue,
        );
        let camera_entity = world.spawn_entity();
        let mut cameras = CameraStore::new();
        cameras.insert(camera_entity.index, Arc::new(RwLock::new(camera)));
        world.insert_resource(cameras);
        world.insert_resource(ActiveCamera(camera_entity));

        let scene = RenderScene::from_world(&world);
        for _ in 0..2 {
            renderer.render(
                &target,
                &scene,
                &device,
                &queue,
                None,
                &std::collections::HashSet::new(),
                None,
            );
        }
    }

    #[test]
    fn renderer_renders_with_smaa_on_noop_backend() {
        let (device, queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor::default());
        let mut renderer = Renderer::new(
            TextureFormat::Bgra8UnormSrgb,
            Vector2::new(320, 180),
            &device,
            &queue,
        );
        renderer.set_anti_aliasing(AntiAliasing::Smaa);

        let target = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Renderer noop target"),
                size: wgpu::Extent3d {
                    width: 320,
                    height: 180,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&Default::default());

        let world = orbital_ecs::World::new();
        let scene = RenderScene::from_world(&world);
        renderer.render(
            &target,
            &scene,
            &device,
            &queue,
            None,
            &std::collections::HashSet::new(),
            None,
        );
    }
}
