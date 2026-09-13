use orbital::app::{App, AppSettings, Module};
use orbital::ecs::{System, World};
use orbital::ecs_bridge::SurfaceFormatResource;
use orbital::logging::{self, error, info};
use orbital::twod::{ShapeDescriptor, Vertex2D, Batch2D};
use orbital::app::{RenderOverlay, RenderOverlayContext, RenderOverlayResource};
use orbital::renderer::{Camera2DUniform, Renderer2D};

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
        name: "2D Scene".to_string(),
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(Scene2DModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

/// Overlay that renders 2D shapes.
struct ShapeOverlay {
    renderer: Renderer2D,
    camera_buffer: orbital::wgpu::Buffer,
    camera_bind_group: orbital::wgpu::BindGroup,
    vertices: Vec<Vertex2D>,
}

impl ShapeOverlay {
    fn new(device: &orbital::wgpu::Device, format: orbital::wgpu::TextureFormat) -> Self {
        let renderer = Renderer2D::new(device, format);

        let camera_buffer = device.create_buffer(&orbital::wgpu::BufferDescriptor {
            label: Some("2D Camera Buffer"),
            size: std::mem::size_of::<Camera2DUniform>() as u64,
            usage: orbital::wgpu::BufferUsages::UNIFORM | orbital::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = renderer.create_bind_group(device, &camera_buffer);

        Self {
            renderer,
            camera_buffer,
            camera_bind_group,
            vertices: Vec::new(),
        }
    }
}

impl RenderOverlay for ShapeOverlay {
    fn render(&mut self, ctx: RenderOverlayContext) {
        if self.vertices.is_empty() {
            return;
        }

        let (screen_w, screen_h) = ctx.screen_size;

        // Create orthographic projection: pixel coordinates to NDC
        let projection = [
            [2.0 / screen_w, 0.0, 0.0, 0.0],
            [0.0, -2.0 / screen_h, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        let uniform = Camera2DUniform {
            view_proj: projection,
            screen_size: [screen_w, screen_h],
            _padding: [0.0; 2],
        };

        // Convert uniform to bytes
        let uniform_bytes = unsafe {
            std::slice::from_raw_parts(
                &uniform as *const Camera2DUniform as *const u8,
                std::mem::size_of::<Camera2DUniform>(),
            )
        };

        ctx.queue
            .write_buffer(&self.camera_buffer, 0, uniform_bytes);

        // Render using the 2D renderer directly
        let mut command_encoder = ctx.device.create_command_encoder(&orbital::wgpu::CommandEncoderDescriptor {
            label: Some("2D Scene Encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&orbital::wgpu::RenderPassDescriptor {
                label: Some("2D Scene Pass"),
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
                    self.vertices.as_ptr() as *const u8,
                    self.vertices.len() * std::mem::size_of::<Vertex2D>(),
                )
            };
            ctx.queue.write_buffer(self.renderer.vertex_buffer(), 0, byte_data);

            render_pass.set_pipeline(self.renderer.pipeline());
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer().slice(..));
            render_pass.draw(0..self.vertices.len() as u32, 0..1);
        }

        ctx.queue.submit(std::iter::once(command_encoder.finish()));
    }
}

struct Scene2DModule;

impl Module for Scene2DModule {
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

        // Create shapes and generate vertices
        let mut batch = Batch2D::new();

        // Red square at (100, 100) with size 100x100
        batch.push_shape(&orbital::twod::shape::generate_rect(
            100.0, 100.0, 100.0, 100.0, [0.9, 0.3, 0.2, 1.0],
        ));

        // Green circle at (300, 200) with radius 50
        batch.push_shape(&orbital::twod::shape::generate_circle(
            [300.0, 200.0],
            50.0,
            32,
            [0.2, 0.8, 0.3, 1.0],
        ));

        // Blue triangle at (500, 150) - generate at origin then offset
        let triangle = ShapeDescriptor::solid_triangle([0.3, 0.4, 0.9, 1.0]);
        let mut tri_verts = orbital::twod::shape::generate_shape_vertices(&triangle, 80.0, 80.0);
        for v in &mut tri_verts {
            v.position[0] += 500.0;
            v.position[1] += 150.0;
        }
        batch.push_shape(&tri_verts);

        // Yellow quad at (200, 350) with size 150x80
        batch.push_shape(&orbital::twod::shape::generate_rect(
            200.0, 350.0, 150.0, 80.0, [1.0, 1.0, 0.2, 1.0],
        ));

        // Cyan polygon (hexagon) at (600, 300)
        let hex = ShapeDescriptor::solid_polygon(6, [0.2, 0.8, 0.8, 1.0]);
        let mut hex_verts = orbital::twod::shape::generate_shape_vertices(&hex, 60.0, 60.0);
        for v in &mut hex_verts {
            v.position[0] += 600.0;
            v.position[1] += 300.0;
        }
        batch.push_shape(&hex_verts);

        info!("Created 2D scene with {} vertices", batch.vertex_count());

        // Create overlay and set vertices
        let mut overlay = ShapeOverlay::new(device, format);
        overlay.vertices = batch.vertices;

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
