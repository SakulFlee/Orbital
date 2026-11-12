//! Generic render-overlay hook — lets modules inject post‑main‑pass drawing
//! (e.g. debug wireframes, gizmos, HUD) without modifying the engine runtime.

use std::time::Duration;

use crate::render_layer::RenderLayer;

/// Optional CPU timings reported by a layer renderer.
///
/// The runtime aggregates these into the `iced draw` / `iced submit` frame
/// stages. Renderers that do not report timings leave both values at zero.
#[derive(Debug, Clone, Copy, Default)]
pub struct OverlayRenderTimings {
    /// Time spent preparing/drawing the overlay (before submission).
    pub draw: Duration,
    /// Time spent recording or submitting the overlay's command buffer.
    ///
    /// Layers that share the main submission measure command recording here;
    /// the runtime's scene submission performs the actual `queue.submit`.
    pub submit: Duration,
}

/// Context passed to [`LayerRenderer::render`].
pub struct RenderOverlayContext<'a> {
    /// Colour attachment to draw over.
    pub target_view: &'a wgpu::TextureView,
    /// GPU camera uniform buffer (binding 0 of the world bind group).
    pub camera_buffer: &'a wgpu::Buffer,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    /// Shared encoder for overlays that record into the main scene encoder.
    /// Layer renderers that record their own command buffer (for example iced)
    /// receive `None`.
    pub encoder: Option<&'a mut wgpu::CommandEncoder>,
    /// Read‑only access to the ECS world so the overlay can query
    /// camera, models, instances, etc.
    pub ecs: &'a orbital_ecs::World,
    /// Screen dimensions (width, height) in pixels.
    pub screen_size: (f32, f32),
}

/// How a layer renderer submits its work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlaySubmission {
    /// Record into the main scene encoder; no extra submit.
    MainEncoder,
    /// Record an own command buffer and merge it into the main submission.
    ///
    /// The command buffer is submitted after the scene's command buffer, so
    /// the layer still draws on top of the scene, but the driver only handles
    /// one `queue.submit` call for the whole frame. This is the preferred mode
    /// for overlays whose renderer exposes an encoder, such as iced.
    MainSubmit,
    /// Own a command buffer and submit it after the main scene submission.
    ///
    /// Owned-submission layers always run after all main-encoder and
    /// main-submit layers, regardless of `RenderLayer` order.
    Owned,
}

/// A layer-aware render overlay that draws at a specific layer.
///
/// The runtime sorts layer renderers by layer order. Renderers that use
/// [`OverlaySubmission::MainEncoder`] receive the shared encoder in
/// [`RenderOverlayContext::encoder`]; renderers that record their own command
/// buffer receive `None`.
pub trait LayerRenderer: Send + Sync {
    /// Returns the layer this renderer should be rendered in.
    fn layer(&self) -> RenderLayer;

    /// Human-readable name used by the performance detail log.
    fn debug_name(&self) -> &str {
        self.layer().name()
    }

    /// Whether this renderer is currently visible.
    ///
    /// Used for capture metadata only. A renderer that hides itself through
    /// ECS state (for example an iced panel) should override this.
    fn is_visible(&self, _ecs: &orbital_ecs::World) -> bool {
        true
    }

    /// Selects whether this layer records into the main encoder or owns a
    /// separate submission.
    fn submission(&self) -> OverlaySubmission {
        OverlaySubmission::MainEncoder
    }

    /// Renders the overlay.
    ///
    /// Called for [`OverlaySubmission::MainEncoder`] layers with the shared
    /// encoder, and for [`OverlaySubmission::Owned`] layers without one.
    /// [`OverlaySubmission::MainSubmit`] layers use
    /// [`render_main_submit`](Self::render_main_submit) instead.
    fn render(&mut self, ctx: RenderOverlayContext);

    /// Records this layer into command buffers for the main scene submission.
    ///
    /// Called only for [`OverlaySubmission::MainSubmit`] layers, after the
    /// main-encoder layers have been recorded. The returned buffers are
    /// submitted after the scene command buffer, so the layer still draws on
    /// top of the scene. The runtime calls
    /// [`finish_main_submit`](Self::finish_main_submit) once the submission has
    /// been issued.
    fn render_main_submit(&mut self, _ctx: RenderOverlayContext) -> Vec<wgpu::CommandBuffer> {
        Vec::new()
    }

    /// Releases any state tied to the command buffers returned by
    /// [`render_main_submit`](Self::render_main_submit).
    ///
    /// Called after the runtime has submitted the main scene command buffer
    /// and the layer's own buffers.
    fn finish_main_submit(&mut self) {}

    /// Process input events and update capture state (no GPU rendering).
    ///
    /// Called from `ModuleRuntime::update()` **before** game systems run so
    /// that touch-capture information (e.g. `IcedCapturedTouches`) is current
    /// when game touch input is processed.  The default implementation is a
    /// no-op — only renderers that need per-frame event processing (like
    /// iced UI panels) should override this.
    fn process_events(&mut self, _ecs: &mut orbital_ecs::World) {}

    /// Returns and clears the CPU timings from the last [`render`](Self::render)
    /// call. Renderers that do not measure themselves return zeroes.
    fn take_render_timings(&mut self) -> OverlayRenderTimings {
        OverlayRenderTimings::default()
    }
}

/// ECS resource — insert this into the world to activate render overlays.
///
/// `ModuleRuntime::redraw()` checks for this resource and renders registered
/// layers in `RenderLayer` order. Main-encoder layers are recorded before the
/// scene submission; main-submit layers append their command buffers to that
/// submission; owned-submission layers run afterwards.
pub struct RenderOverlayResource {
    /// Layer-aware renderers (sorted by layer order before rendering).
    pub layer_renderers: std::sync::Mutex<Vec<Box<dyn LayerRenderer>>>,
}

impl RenderOverlayResource {
    pub fn new() -> Self {
        Self {
            layer_renderers: std::sync::Mutex::new(Vec::new()),
        }
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
