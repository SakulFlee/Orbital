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

/// ECS resource — insert this into the world to activate render overlays.
///
/// `ModuleRuntime::redraw()` checks for this resource after the main render
/// pass and calls [`RenderOverlay::render`] on each registered overlay, in
/// insertion order.
pub struct RenderOverlayResource(pub std::sync::Mutex<Vec<Box<dyn RenderOverlay>>>);

impl RenderOverlayResource {
    pub fn new() -> Self {
        Self(std::sync::Mutex::new(Vec::new()))
    }

    /// Register an overlay to be drawn after the main scene pass.
    ///
    /// Multiple modules can call this on the same resource; overlays render in
    /// the order they were added.
    pub fn add(&self, overlay: Box<dyn RenderOverlay>) {
        self.0.lock().unwrap().push(overlay);
    }
}

impl Default for RenderOverlayResource {
    fn default() -> Self {
        Self::new()
    }
}
