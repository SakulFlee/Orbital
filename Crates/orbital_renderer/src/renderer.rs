use std::sync::{Arc, RwLock};
use std::time::Instant;

use cgmath::Vector2;
use wgpu::{
    BindGroup, BindGroupEntry, Color, CommandEncoder, CommandEncoderDescriptor, CompareFunction,
    Device, FilterMode, IndexFormat, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, SamplerDescriptor, StoreOp,
    TextureFormat, TextureView, TextureViewDimension,
};

use orbital_camera::{Camera, CameraDescriptor};
use orbital_cull::CullResources;
use orbital_ibl_brdf::IblBrdf;
use orbital_material_shader::MaterialShader;
use orbital_model::Model;
use orbital_shadow::{ShadowGpuData, ShadowRenderer};
use orbital_texture::Texture;
use orbital_world_environment::{GeneratedSkyParameters, WorldEnvironment};

use crate::scene::RenderScene;

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
        let depth_view = depth_texture.create_view(&Default::default());

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
    depth_texture: Texture,
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
        let depth_texture = Texture::depth_texture(&resolution, device, queue);

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

        Self {
            surface_texture_format,
            depth_texture,
            shadow_renderer: None,
            fallbacks: RendererFallbacks::new(device, queue),
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
        _device: &Device,
        _queue: &Queue,
    ) {
        self.surface_texture_format = surface_texture_format;
    }

    pub fn change_resolution(&mut self, resolution: Vector2<u32>, device: &Device, queue: &Queue) {
        self.depth_texture = Texture::depth_texture(&resolution, device, queue);
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

    pub fn render(
        &mut self,
        target_view: &TextureView,
        scene: &RenderScene,
        device: &Device,
        queue: &Queue,
        cull: Option<&CullResources>,
        dirty_set: &std::collections::HashSet<u32>,
    ) {
        let ms = |duration: std::time::Duration| duration.as_secs_f64() * 1000.0;
        let render_start = Instant::now();

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Orbital::Render::Encoder"),
        });

        // Write timestamp 0: start of shadow pass
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 0);
        }

        // Shadow pass (before main passes)
        if let Some(sr) = self.shadow_renderer.as_mut()
            && let Some(camera) = &scene.camera
        {
            sr.render(
                &mut command_encoder,
                &scene.models,
                &scene.shadow_lights,
                &camera.perspective_view_projection,
                camera.near,
                camera.far,
                device,
                queue,
                dirty_set,
            );
        }
        let shadow_end = Instant::now();

        // Write timestamp 1: shadow done, skybox start
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 1);
        }

        // Build (or reuse) the bind group after the shadow pass: shadow
        // texture reallocation during that pass can change its resource views.
        let world_bind_group = self.world_bind_group(scene, device, queue);

        if let Some(world_environment) = scene.environment.as_deref() {
            self.render_sky_box(
                target_view,
                world_environment.material_shader(),
                &world_bind_group,
                &mut command_encoder,
            );
        }
        let skybox_end = Instant::now();

        // Write timestamp 2: skybox done, models start
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 2);
        }

        self.render_models(
            &scene.models,
            target_view,
            &world_bind_group,
            &mut command_encoder,
            cull,
        );
        let models_end = Instant::now();

        // Write timestamp 3: main pass done
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 3);
        }

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
            shadow_ms: ms(shadow_end - render_start),
            skybox_ms: ms(skybox_end - shadow_end),
            models_ms: ms(models_end - skybox_end),
            encode_ms: ms(submit_start - render_start),
            submit_ms,
            timestamp_readback_ms: ms(timestamp_readback_start.elapsed()),
        };
    }

    fn render_sky_box(
        &self,
        target_view: &TextureView,
        sky_box_shader: &MaterialShader,
        world_bind_group: &BindGroup,
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

        render_pass.set_pipeline(sky_box_shader.pipeline());
        render_pass.set_bind_group(0, world_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn render_models(
        &self,
        models: &[Arc<Model>],
        target_view: &TextureView,
        world_bind_group: &BindGroup,
        command_encoder: &mut CommandEncoder,
        cull: Option<&CullResources>,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Model RenderPass"),
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
                view: self.depth_texture.view(),
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

        for (i, model) in models.iter().enumerate() {
            for material in model.materials() {
                render_pass.set_pipeline(material.pipeline());
                render_pass.set_bind_group(0, world_bind_group, &[]);
                render_pass.set_bind_group(1, material.bind_group(), &[]);
                render_pass.set_vertex_buffer(0, model.mesh().vertex_buffer().slice(..));

                if let Some(cr) = cull {
                    // CPU-culled: read from filtered output at model offset
                    let byte_off = cr.model_first_instance(i) as u64 * 64;
                    render_pass
                        .set_vertex_buffer(1, cr.filtered_instance_buffer().slice(byte_off..));
                    render_pass.set_index_buffer(
                        model.mesh().index_buffer().slice(..),
                        IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(
                        0..model.mesh().index_count(),
                        0,
                        0..cr.visible_count(i),
                    );
                } else {
                    // No culling data: draw all instances from the original buffer
                    render_pass.set_vertex_buffer(1, model.instance_buffer().slice(..));
                    render_pass.set_index_buffer(
                        model.mesh().index_buffer().slice(..),
                        IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed(
                        0..model.mesh().index_count(),
                        0,
                        0..model.instance_count(),
                    );
                }
            }
        }
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
