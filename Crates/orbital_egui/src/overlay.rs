use orbital_app::{RenderOverlay, RenderOverlayContext};

use crate::EguiState;

/// [`RenderOverlay`] implementation that draws the egui debug UI.
///
/// Created by [`EguiModule`](crate::EguiModule) and inserted as a
/// [`RenderOverlayResource`](orbital_app::RenderOverlayResource).
pub struct EguiOverlay {
    pub(crate) egui_ctx: egui::Context,
    pub(crate) winit_state: Option<egui_winit::State>,
    pub(crate) wgpu_renderer: egui_wgpu::Renderer,
    pub(crate) panels: Vec<Box<dyn crate::ui::Panel>>,
    /// Whether deferred initialization has completed (scale factor, etc.).
    pub(crate) initialized: bool,
}

impl RenderOverlay for EguiOverlay {
    fn render(&mut self, ctx: RenderOverlayContext) {
        // Check if egui is enabled
        let enabled = ctx
            .ecs
            .get_resource::<EguiState>()
            .map(|s| s.enabled)
            .unwrap_or(false);
        if !enabled {
            return;
        }

        // Deferred initialization: create winit_state on first frame when we have the window
        if self.winit_state.is_none() {
            let max_texture_side = ctx.device.limits().max_texture_dimension_2d as usize;
            self.winit_state = Some(egui_winit::State::new(
                self.egui_ctx.clone(),
                egui::ViewportId::ROOT,
                ctx.window,
                Some(ctx.window.scale_factor() as f32),
                None, // theme
                Some(max_texture_side),
            ));
            self.initialized = true;
        }

        let winit_state = self.winit_state.as_mut().unwrap();

        // Collect raw input from egui_winit
        let raw_input = winit_state.take_egui_input(ctx.window);

        // Run egui frame
        let full_output = self.egui_ctx.run_ui(raw_input, |ui| {
            for panel in &mut self.panels {
                panel.ui(ui);
            }
        });

        // Handle platform output (cursor icons, clipboard, etc.)
        winit_state.handle_platform_output(
            ctx.window,
            full_output.platform_output,
        );

        // Tessellate
        let pixels_per_point = ctx.window.scale_factor() as f32;
        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, pixels_per_point);

        // Build screen descriptor for wgpu_renderer
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                ctx.window.inner_size().width,
                ctx.window.inner_size().height,
            ],
            pixels_per_point,
        };

        // Render
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("egui encoder"),
            });

        // Upload textures
        for (id, delta) in &full_output.textures_delta.set {
            self.wgpu_renderer
                .update_texture(ctx.device, ctx.queue, *id, delta);
        }

        // Update buffers
        self.wgpu_renderer.update_buffers(
            ctx.device,
            ctx.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        // Render pass
        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: ctx.target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        self.wgpu_renderer.render(
            &mut render_pass.forget_lifetime(),
            &clipped_primitives,
            &screen_descriptor,
        );

        // Free textures
        for id in &full_output.textures_delta.free {
            self.wgpu_renderer.free_texture(id);
        }

        // Submit
        ctx.queue.submit(std::iter::once(encoder.finish()));
    }

    fn on_window_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) {
        if let Some(winit_state) = &mut self.winit_state {
            let _ = winit_state.on_window_event(window, event);
        }
    }
}
