use orbital::app::{App, AppSettings, Module};
use orbital::app::{RenderOverlay, RenderOverlayContext, RenderOverlayResource};
use orbital::ecs::{System, World};
use orbital::ecs_bridge::SurfaceFormatResource;
use orbital::logging::{self, error, info};
use orbital::renderer::TextRenderer;
use orbital::renderer::UiRenderer;
use orbital::text::{FontData, SdfAtlas, TextConfig, generate_text_mesh};
use orbital::twod::Vertex2D;

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(target_os = "android"))]
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let app_settings = AppSettings {
        vsync_enabled: true,
        name: "UI Demo".to_string(),
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(UiDemoModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

/// Overlay that renders UI backgrounds (rounded rectangles).
struct UiBackgroundOverlay {
    renderer: UiRenderer,
    vertices: Vec<Vertex2D>,
}

impl UiBackgroundOverlay {
    fn new(device: &orbital::wgpu::Device, format: orbital::wgpu::TextureFormat) -> Self {
        let renderer = UiRenderer::new(device, format);
        Self {
            renderer,
            vertices: Vec::new(),
        }
    }

    /// Generates vertices for a rounded rectangle.
    fn push_rounded_rect(
        vertices: &mut Vec<Vertex2D>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        corner_radius: f32,
        color: [f32; 4],
    ) {
        // Convert pixel coordinates to NDC
        // We'll do this in the shader instead, so just pass pixel coords
        // The shader expects UV coordinates in [0,1] range

        // Two triangles for the quad
        // UV: (0,0) = top-left, (1,1) = bottom-right
        // shape_params: (corner_radius_normalized, 0)

        let r = corner_radius / (w.min(h) / 2.0); // Normalize corner radius (p ranges 0-1, where 1 = min(w,h)/2 pixels from center)

        // Top-left triangle
        vertices.push(Vertex2D::full([x, y], color, [0.0, 0.0], [r, 0.0]));
        vertices.push(Vertex2D::full([x + w, y], color, [1.0, 0.0], [r, 0.0]));
        vertices.push(Vertex2D::full([x + w, y + h], color, [1.0, 1.0], [r, 0.0]));

        // Bottom-left triangle
        vertices.push(Vertex2D::full([x, y], color, [0.0, 0.0], [r, 0.0]));
        vertices.push(Vertex2D::full([x + w, y + h], color, [1.0, 1.0], [r, 0.0]));
        vertices.push(Vertex2D::full([x, y + h], color, [0.0, 1.0], [r, 0.0]));
    }
}

impl RenderOverlay for UiBackgroundOverlay {
    fn render(&mut self, ctx: RenderOverlayContext) {
        if self.vertices.is_empty() {
            return;
        }

        let (screen_w, screen_h) = ctx.screen_size;

        // The shader expects NDC coordinates
        // We need to transform pixel coordinates to NDC
        let mut ndc_vertices = Vec::with_capacity(self.vertices.len());
        for v in &self.vertices {
            let ndc_x = (v.position[0] / screen_w) * 2.0 - 1.0;
            let ndc_y = 1.0 - (v.position[1] / screen_h) * 2.0; // Flip Y
            ndc_vertices.push(Vertex2D::full(
                [ndc_x, ndc_y],
                v.color,
                v.texcoord,
                v.shape_params,
            ));
        }

        let mut command_encoder =
            ctx.device
                .create_command_encoder(&orbital::wgpu::CommandEncoderDescriptor {
                    label: Some("UI Background Encoder"),
                });

        {
            let mut render_pass =
                command_encoder.begin_render_pass(&orbital::wgpu::RenderPassDescriptor {
                    label: Some("UI Background Pass"),
                    color_attachments: &[Some(orbital::wgpu::RenderPassColorAttachment {
                        view: ctx.target_view,
                        resolve_target: None,
                        ops: orbital::wgpu::Operations {
                            load: orbital::wgpu::LoadOp::Load,
                            store: orbital::wgpu::StoreOp::Store,
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
                    ndc_vertices.as_ptr() as *const u8,
                    ndc_vertices.len() * std::mem::size_of::<Vertex2D>(),
                )
            };
            ctx.queue
                .write_buffer(self.renderer.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(self.renderer.pipeline());
            render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer().slice(..));
            render_pass.draw(0..ndc_vertices.len() as u32, 0..1);
        }

        ctx.queue.submit(std::iter::once(command_encoder.finish()));
    }
}

/// Overlay that renders text labels for UI elements.
struct TextOverlay {
    renderer: TextRenderer,
    font: FontData,
    atlas: Option<SdfAtlas>,
    texts: Vec<(String, [f32; 2], f32, [f32; 4])>, // (text, position, font_size, color)
}

impl TextOverlay {
    fn new(device: &orbital::wgpu::Device, format: orbital::wgpu::TextureFormat) -> Self {
        let font = orbital::text::DefaultFont::load();
        let renderer = TextRenderer::new(device, format);

        Self {
            renderer,
            font,
            atlas: None,
            texts: Vec::new(),
        }
    }

    /// Builds the SDF atlas for the characters we need.
    fn build_atlas(&mut self) {
        // Character set for UI labels
        let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 .,!?-:Enter usernamePlayGameExitIagreetotheterms";
        self.atlas = Some(SdfAtlas::build_atlas(
            &mut self.font,
            24,
            chars,
            1.0,
            1024,
            1024,
        ));
    }

    /// Uploads the atlas to GPU.
    fn upload_atlas(&mut self, device: &orbital::wgpu::Device, queue: &orbital::wgpu::Queue) {
        if let Some(atlas) = &self.atlas {
            self.renderer.upload_atlas(device, queue, atlas);
        }
    }

    /// Adds a text to render.
    fn add_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: [f32; 4]) {
        self.texts
            .push((text.to_string(), [x, y], font_size, color));
    }

    /// Generates all text vertices.
    fn generate_vertices(&self) -> Vec<Vertex2D> {
        let mut all_vertices = Vec::new();

        for (text, pos, font_size, color) in &self.texts {
            let config = TextConfig {
                font_size: *font_size,
                color: *color,
                ..Default::default()
            };

            let mut vertices = generate_text_mesh(text, &self.font, &config, self.atlas.as_ref());

            // Offset vertices by position
            for v in &mut vertices {
                v.position[0] += pos[0];
                v.position[1] += pos[1];
            }

            all_vertices.extend(vertices);
        }

        all_vertices
    }
}

impl RenderOverlay for TextOverlay {
    fn render(&mut self, ctx: RenderOverlayContext) {
        // Build and upload atlas on first render
        if !self.renderer.has_atlas() {
            self.build_atlas();
            self.upload_atlas(ctx.device, ctx.queue);
        }

        let vertices = self.generate_vertices();
        if vertices.is_empty() {
            return;
        }

        let (screen_w, screen_h) = ctx.screen_size;

        // Convert to NDC
        let mut ndc_vertices = Vec::with_capacity(vertices.len());
        for v in &vertices {
            let ndc_x = (v.position[0] / screen_w) * 2.0 - 1.0;
            let ndc_y = 1.0 - (v.position[1] / screen_h) * 2.0;
            ndc_vertices.push(Vertex2D::full(
                [ndc_x, ndc_y],
                v.color,
                v.texcoord,
                v.shape_params,
            ));
        }

        // Create SDF params uniform
        let sdf_params = orbital::renderer::SdfParamsUniform {
            smoothing: 0.04,
            outline_width: 0.0,
            _padding: [0.0; 2],
            outline_color: [0.0, 0.0, 0.0, 1.0],
        };

        let sdf_params_buffer = ctx.device.create_buffer(&orbital::wgpu::BufferDescriptor {
            label: Some("SDF Params Buffer"),
            size: std::mem::size_of::<orbital::renderer::SdfParamsUniform>() as u64,
            usage: orbital::wgpu::BufferUsages::UNIFORM | orbital::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Write uniform data
        let uniform_bytes = unsafe {
            std::slice::from_raw_parts(
                &sdf_params as *const orbital::renderer::SdfParamsUniform as *const u8,
                std::mem::size_of::<orbital::renderer::SdfParamsUniform>(),
            )
        };
        ctx.queue.write_buffer(&sdf_params_buffer, 0, uniform_bytes);

        // Create bind groups
        let atlas_bind_group = match self.renderer.create_atlas_bind_group(ctx.device) {
            Some(bg) => bg,
            None => return,
        };

        let params_bind_group = ctx
            .device
            .create_bind_group(&orbital::wgpu::BindGroupDescriptor {
                label: Some("SDF Params Bind Group"),
                layout: &self.renderer.pipeline().get_bind_group_layout(2),
                entries: &[orbital::wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sdf_params_buffer.as_entire_binding(),
                }],
            });

        let mut command_encoder =
            ctx.device
                .create_command_encoder(&orbital::wgpu::CommandEncoderDescriptor {
                    label: Some("Text Encoder"),
                });

        {
            let mut render_pass =
                command_encoder.begin_render_pass(&orbital::wgpu::RenderPassDescriptor {
                    label: Some("Text Pass"),
                    color_attachments: &[Some(orbital::wgpu::RenderPassColorAttachment {
                        view: ctx.target_view,
                        resolve_target: None,
                        ops: orbital::wgpu::Operations {
                            load: orbital::wgpu::LoadOp::Load,
                            store: orbital::wgpu::StoreOp::Store,
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
                    ndc_vertices.as_ptr() as *const u8,
                    ndc_vertices.len() * std::mem::size_of::<Vertex2D>(),
                )
            };
            ctx.queue
                .write_buffer(self.renderer.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(self.renderer.pipeline());
            render_pass.set_bind_group(1, &atlas_bind_group, &[]);
            render_pass.set_bind_group(2, &params_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer().slice(..));
            render_pass.draw(0..ndc_vertices.len() as u32, 0..1);
        }

        ctx.queue.submit(std::iter::once(command_encoder.finish()));
    }
}

struct UiDemoModule;

impl Module for UiDemoModule {
    fn setup(
        &self,
        ecs: &mut World,
        device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Get surface format
        let format = ecs
            .get_resource::<SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(orbital::wgpu::TextureFormat::Bgra8UnormSrgb);

        // Create UI overlay
        let mut overlay = UiBackgroundOverlay::new(device, format);

        // Generate UI background vertices
        let bg_color = [0.2, 0.2, 0.2, 0.9];
        let btn_color = [0.3, 0.3, 0.3, 0.9];
        let accent_color = [0.2, 0.5, 0.8, 0.9];

        // Title background
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            50.0,
            20.0,
            500.0,
            50.0,
            8.0,
            bg_color,
        );

        // Play button
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0,
            100.0,
            200.0,
            50.0,
            8.0,
            accent_color,
        );

        // Text box background
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0,
            180.0,
            250.0,
            40.0,
            4.0,
            [0.15, 0.15, 0.15, 0.9],
        );

        // Checkbox background
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0,
            240.0,
            30.0,
            30.0,
            4.0,
            btn_color,
        );

        // Exit button
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0,
            300.0,
            200.0,
            50.0,
            8.0,
            btn_color,
        );

        info!("Created UI demo with {} vertices", overlay.vertices.len());

        // Create text overlay with labels
        let mut text_overlay = TextOverlay::new(device, format);

        // Title text
        text_overlay.add_text("UI Demo", 60.0, 35.0, 24.0, [1.0, 1.0, 1.0, 1.0]);

        // Play button text
        text_overlay.add_text("Play Game", 150.0, 115.0, 20.0, [1.0, 1.0, 1.0, 1.0]);

        // Text box placeholder
        text_overlay.add_text(
            "Enter username...",
            110.0,
            195.0,
            16.0,
            [0.5, 0.5, 0.5, 1.0],
        );

        // Checkbox label
        text_overlay.add_text(
            "I agree to the terms",
            140.0,
            248.0,
            14.0,
            [0.8, 0.8, 0.8, 1.0],
        );

        // Exit button text
        text_overlay.add_text("Exit", 180.0, 315.0, 20.0, [1.0, 1.0, 1.0, 1.0]);

        // Register overlays
        if ecs.get_resource::<RenderOverlayResource>().is_none() {
            ecs.insert_resource(RenderOverlayResource::new());
        }
        if let Some(res) = ecs.get_resource_mut::<RenderOverlayResource>() {
            res.add(Box::new(overlay));
            res.add(Box::new(text_overlay));
        }

        vec![]
    }
}
