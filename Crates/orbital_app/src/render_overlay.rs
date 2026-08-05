//! Generic render-overlay hook — lets modules inject post‑main‑pass drawing
//! (e.g. debug wireframes, gizmos, HUD) without modifying the engine runtime.

/// Context passed to [`RenderOverlay::render`].
pub struct RenderOverlayContext<'a> {
    /// Colour attachment to draw over.
    pub target_view: &'a wgpu::TextureView,
    /// GPU camera uniform buffer (binding 0 of the world bind group).
    pub camera_buffer: &'a wgpu::Buffer,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    /// Read‑only access to the ECS world so the overlay can query
    /// camera, models, instances, etc.
    pub ecs: &'a orbital_ecs::World,
    /// Reference to the application window.
    ///
    /// Useful for overlays that need screen dimensions, scale factor,
    /// or need to interact with the windowing system (e.g. egui).
    pub window: &'a winit::window::Window,
}

/// A render‑overlay that draws after the main scene pass.
///
/// Implementations create their own command encoder, begin a separate render
/// pass with `LoadOp::Load` (preserving the rendered frame), issue draw calls,
/// and submit the encoder.
pub trait RenderOverlay: Send + Sync {
    fn render(&mut self, ctx: RenderOverlayContext);

    /// Called for each [`WindowEvent`](winit::event::WindowEvent) before the
    /// engine processes it. Override this to forward events to a UI library
    /// (e.g. egui-winit). The default implementation does nothing.
    fn on_window_event(
        &mut self,
        _window: &winit::window::Window,
        _event: &winit::event::WindowEvent,
    ) {
    }
}

/// ECS resource — insert this into the world to activate an overlay.
///
/// `ModuleRuntime::redraw()` checks for this resource after the main render
/// pass and calls [`RenderOverlay::render`] on it.
pub struct RenderOverlayResource(pub std::sync::Mutex<Box<dyn RenderOverlay>>);
