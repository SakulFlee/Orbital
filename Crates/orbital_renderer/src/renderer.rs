use cgmath::{Matrix4, Vector2};
use wgpu::{
    BindGroup, Color, CommandEncoder, CommandEncoderDescriptor, Device, IndexFormat, LoadOp,
    Operations, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

use orbital_resources::{
    CullResources, MaterialShader, Model, ShadowLightInfo, ShadowRenderer, Texture,
    WorldEnvironment,
};

pub struct Renderer {
    surface_texture_format: TextureFormat,
    depth_texture: Texture,
    shadow_renderer: Option<ShadowRenderer>,
    timestamp_query_set: Option<wgpu::QuerySet>,
    timestamp_resolve_buffer: wgpu::Buffer,
    timestamp_staging_bufs: [wgpu::Buffer; 2],
    timestamp_read_frame: usize,
    prev_gpu_ns: [f64; 3],
    prev_resolve_sub: Option<wgpu::SubmissionIndex>,
}

impl Renderer {
    pub fn surface_texture_format(&self) -> &TextureFormat {
        &self.surface_texture_format
    }

    pub fn prev_gpu_ns(&self) -> [f64; 3] {
        self.prev_gpu_ns
    }
}

const TS_COUNT: u32 = 3; // 0=shadow_start, 1=skybox, 2=main_end
const TS_BUF_SIZE: u64 = TS_COUNT as u64 * 8; // 24 bytes

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
            timestamp_query_set,
            timestamp_resolve_buffer,
            timestamp_staging_bufs,
            timestamp_read_frame: 0,
            prev_gpu_ns: [0.0; 3],
            prev_resolve_sub: None,
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

    pub fn render(
        &mut self,
        target_view: &TextureView,
        world_bind_group: &BindGroup,
        world_environment_option: Option<&WorldEnvironment>,
        models: Vec<&Model>,
        device: &Device,
        queue: &Queue,
        cull: Option<&CullResources>,
        shadow_lights: &[ShadowLightInfo],
        camera_perspective_view_proj: Option<&Matrix4<f32>>,
        camera_near: f32,
        camera_far: f32,
        dirty_set: &std::collections::HashSet<u32>,
    ) {
        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Orbital::Render::Encoder"),
        });

        // Single-encoder cull mode (`ORBITAL_CULL_SINGLE_ENCODER`): dispatch
        // the GPU cull compute here, at the top of the render submission, so
        // the storage→vertex/indirect buffer transitions are tracked within a
        // single submit. This sidesteps cross-submission barrier gaps that
        // some drivers (notably Adreno) exhibit when the cull compute is
        // submitted separately from the pass that reads its output.
        if let Some(cr) = cull
            && cr.single_encoder()
        {
            cr.dispatch_into_render(&mut command_encoder);
        }

        // Write timestamp 0: start of shadow pass
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 0);
        }

        // Shadow pass (before main passes)
        if let Some(sr) = self.shadow_renderer.as_mut()
            && let Some(pvp) = camera_perspective_view_proj
        {
            sr.render(
                &mut command_encoder,
                &models,
                shadow_lights,
                pvp,
                camera_near,
                camera_far,
                device,
                queue,
                dirty_set,
            );
        }

        // Write timestamp 1: shadow done, skybox start
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 1);
        }

        if let Some(world_environment) = world_environment_option {
            let sky_box_shader = world_environment.material_shader();
            self.render_sky_box(
                target_view,
                sky_box_shader,
                world_bind_group,
                &mut command_encoder,
            );
        }

        self.render_models(
            models,
            target_view,
            world_bind_group,
            &mut command_encoder,
            cull,
        );

        // Write timestamp 2: main pass done
        if let Some(qs) = &self.timestamp_query_set {
            command_encoder.write_timestamp(qs, 2);
        }

        queue.submit(vec![command_encoder.finish()]);

        // Resolve timestamps into the resolve buffer, then copy into the current staging buffer.
        // Read the OTHER staging buffer from the previous frame (double-buffered to avoid stalls).
        // We wait only on the PREVIOUS frame's resolve submission (not the current frame's render),
        // so CPU/GPU overlap is preserved and the harness doesn't distort FPS measurements.
        if self.timestamp_query_set.is_some() {
            let cur = self.timestamp_read_frame & 1;
            let prev = 1 - cur;

            // Resolve this frame's queries into resolve buffer, then copy to staging[cur]
            let mut resolve_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Orbital::TS_Resolve"),
            });
            resolve_encoder.resolve_query_set(
                self.timestamp_query_set.as_ref().unwrap(),
                0..TS_COUNT,
                &self.timestamp_resolve_buffer,
                0,
            );
            resolve_encoder.copy_buffer_to_buffer(
                &self.timestamp_resolve_buffer,
                0,
                &self.timestamp_staging_bufs[cur],
                0,
                TS_BUF_SIZE,
            );
            let resolve_sub = queue.submit(vec![resolve_encoder.finish()]);

            // Read the PREVIOUS frame's staging buffer (double-buffered: it's from 2 frames ago, so done).
            if let Some(prev_sub) = self.prev_resolve_sub.take() {
                let _ = prev_sub; // unused without blocking poll

                // Map the PREVIOUS frame's staging buffer to read back its results
                let prev_buf = &self.timestamp_staging_bufs[prev];
                let prev_slice = prev_buf.slice(..);
                let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let done2 = done.clone();
                prev_slice.map_async(wgpu::MapMode::Read, move |_| {
                    done2.store(true, std::sync::atomic::Ordering::Relaxed);
                });
                let _ = device.poll(wgpu::PollType::Poll);

                if done.load(std::sync::atomic::Ordering::Relaxed)
                    && let Ok(data) = prev_slice.get_mapped_range()
                {
                    let mut ns = [0.0f64; 3];
                    for i in 0..3 {
                        let bytes: [u8; 8] = data[i * 8..(i + 1) * 8].try_into().unwrap();
                        ns[i] = u64::from_ne_bytes(bytes) as f64;
                    }
                    self.prev_gpu_ns = ns;
                }
                prev_buf.unmap();
            }

            self.prev_resolve_sub = Some(resolve_sub);
            self.timestamp_read_frame += 1;
        }
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
        models: Vec<&Model>,
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
                    // GPU-culled: read from compacted output at model offset
                    let byte_off = cr.model_first_instance(i) as u64 * 64;
                    render_pass.set_vertex_buffer(1, cr.compacted_vertex_buffer().slice(byte_off..));
                    render_pass.set_index_buffer(
                        model.mesh().index_buffer().slice(..),
                        IndexFormat::Uint32,
                    );
                    render_pass.draw_indexed_indirect(cr.indirect_buffer(), i as u64 * 20);
                } else {
                    // Un-culled: draw all instances from the original buffer
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
}
