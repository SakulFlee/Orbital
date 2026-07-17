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
}

/// A render‑overlay that draws after the main scene pass.
///
/// Implementations create their own command encoder, begin a separate render
/// pass with `LoadOp::Load` (preserving the rendered frame), issue draw calls,
/// and submit the encoder.
pub trait RenderOverlay: Send + Sync {
    fn render(&mut self, ctx: RenderOverlayContext);
}

/// ECS resource — insert this into the world to activate an overlay.
///
/// `ModuleRuntime::redraw()` checks for this resource after the main render
/// pass and calls [`RenderOverlay::render`] on it.
pub struct RenderOverlayResource(pub std::sync::Mutex<Box<dyn RenderOverlay>>);
