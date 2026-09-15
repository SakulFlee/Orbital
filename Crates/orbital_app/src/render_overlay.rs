//! Generic render-overlay hook — lets modules inject post‑main‑pass drawing
//! (e.g. debug wireframes, gizmos, HUD) without modifying the engine runtime.

use crate::render_layer::RenderLayer;

/// Context passed to [`RenderOverlay::render`] and [`LayerRenderer::render`].
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
    /// Screen dimensions (width, height) in pixels.
    pub screen_size: (f32, f32),
}

/// A render‑overlay that draws after the main scene pass.
///
/// Implementations create their own command encoder, begin a separate render
/// pass with `LoadOp::Load` (preserving the rendered frame), issue draw calls,
/// and submit the encoder.
pub trait RenderOverlay: Send + Sync {
    fn render(&mut self, ctx: RenderOverlayContext);
}

/// A layer-aware render overlay that draws at a specific layer.
///
/// This trait extends the concept of `RenderOverlay` by adding layer awareness.
/// The render runtime sorts layer renderers by their layer order before rendering.
pub trait LayerRenderer: Send + Sync {
    /// Returns the layer this renderer should be rendered in.
    fn layer(&self) -> RenderLayer;

    /// Renders the overlay.
    fn render(&mut self, ctx: RenderOverlayContext);
}

/// ECS resource — insert this into the world to activate render overlays.
///
/// `ModuleRuntime::redraw()` checks for this resource after the main render
/// pass and calls [`RenderOverlay::render`] on each registered overlay, in
/// insertion order.
pub struct RenderOverlayResource {
    /// Overlays without layer ordering (rendered after main pass, before layers).
    pub overlays: std::sync::Mutex<Vec<Box<dyn RenderOverlay>>>,
    /// Layer-aware renderers (sorted by layer order before rendering).
    pub layer_renderers: std::sync::Mutex<Vec<Box<dyn LayerRenderer>>>,
}

impl RenderOverlayResource {
    pub fn new() -> Self {
        Self {
            overlays: std::sync::Mutex::new(Vec::new()),
            layer_renderers: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Ensures the resource exists in the ECS world and returns a mutable handle.
    pub fn ensure(ecs: &mut orbital_ecs::World) -> orbital_ecs::ResourceMutHandle<'_, Self> {
        if ecs.get_resource::<Self>().is_none() {
            ecs.insert_resource(Self::new());
        }
        ecs.get_resource_mut::<Self>().unwrap()
    }

    /// Register an overlay to be drawn after the main scene pass.
    ///
    /// Multiple modules can call this on the same resource; overlays render in
    /// the order they were added.
    pub fn add(&self, overlay: Box<dyn RenderOverlay>) {
        self.overlays.lock().unwrap().push(overlay);
    }

    /// Register a layer-aware renderer.
    ///
    /// The runtime will sort renderers by their layer order before rendering.
    pub fn add_layer_renderer(&self, renderer: Box<dyn LayerRenderer>) {
        self.layer_renderers.lock().unwrap().push(renderer);
    }

    /// Returns the layer renderers sorted by layer order.
    pub fn sorted_layer_renderers(&self) -> Vec<(RenderLayer, usize)> {
        let renderers = self.layer_renderers.lock().unwrap();
        renderers
            .iter()
            .enumerate()
            .map(|(idx, r)| (r.layer(), idx))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    }
}

impl Default for RenderOverlayResource {
    fn default() -> Self {
        Self::new()
    }
}
