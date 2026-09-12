use orbital::app::{App, AppSettings, Module};
use orbital::ecs::{System, World};
use orbital::ecs_bridge::SurfaceFormatResource;
use orbital::logging::{self, error, info};
use orbital::app::{RenderOverlay, RenderOverlayContext, RenderOverlayResource};
use orbital::renderer::UiRenderer;
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

        let r = corner_radius / w.min(h); // Normalize corner radius

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

        let mut command_encoder = ctx.device.create_command_encoder(&orbital::wgpu::CommandEncoderDescriptor {
            label: Some("UI Background Encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&orbital::wgpu::RenderPassDescriptor {
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
            ctx.queue.write_buffer(self.renderer.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(self.renderer.pipeline());
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
            50.0, 20.0, 500.0, 50.0, 8.0, bg_color,
        );

        // Play button
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0, 100.0, 200.0, 50.0, 8.0, accent_color,
        );

        // Text box background
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0, 180.0, 250.0, 40.0, 4.0, [0.15, 0.15, 0.15, 0.9],
        );

        // Checkbox background
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0, 240.0, 30.0, 30.0, 4.0, btn_color,
        );

        // Exit button
        UiBackgroundOverlay::push_rounded_rect(
            &mut overlay.vertices,
            100.0, 300.0, 200.0, 50.0, 8.0, btn_color,
        );

        info!("Created UI demo with {} vertices", overlay.vertices.len());

        // Register overlay
        if ecs.get_resource::<RenderOverlayResource>().is_none() {
            ecs.insert_resource(RenderOverlayResource::new());
        }
        if let Some(res) = ecs.get_resource_mut::<RenderOverlayResource>() {
            res.add(Box::new(overlay));
        }

        vec![]
    }
}
