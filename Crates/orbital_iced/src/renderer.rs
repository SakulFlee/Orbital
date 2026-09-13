use crate::state::IcedState;
use orbital_app::render_overlay::{LayerRenderer as LayerRendererTrait, RenderOverlayContext};
use orbital_app::RenderLayer;
use orbital_ecs_bridge::{AdapterResource, SurfaceFormatResource};
use std::sync::Mutex;

pub struct IcedLayerRenderer {
    state: IcedState,
    inner: Mutex<Option<RendererInner>>,
}

struct RendererInner {
    engine: iced_wgpu::Engine,
    renderer: iced_wgpu::Renderer,
}

unsafe impl Send for IcedLayerRenderer {}
unsafe impl Sync for IcedLayerRenderer {}

impl IcedLayerRenderer {
    pub fn new(state: IcedState) -> Self {
        Self {
            state,
            inner: Mutex::new(None),
        }
    }

    pub fn state(&self) -> &IcedState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut IcedState {
        &mut self.state
    }
}

impl LayerRendererTrait for IcedLayerRenderer {
    fn layer(&self) -> RenderLayer {
        RenderLayer::UI
    }

    fn render(&mut self, ctx: RenderOverlayContext) {
        let format = ctx
            .ecs
            .get_resource::<SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);

        // Ensure renderer is initialized
        {
            let mut guard = self.inner.lock().unwrap();
            if guard.is_none() {
                let adapter = match ctx.ecs.get_resource::<AdapterResource>() {
                    Some(a) => a.0.as_ref().clone(),
                    None => {
                        log::warn!("No AdapterResource — skipping iced render");
                        return;
                    }
                };

                let engine = iced_wgpu::Engine::new(
                    &adapter,
                    ctx.device.clone(),
                    ctx.queue.clone(),
                    format,
                    None,
                    iced_graphics::Shell::headless(),
                );

                let renderer = iced_wgpu::Renderer::new(engine.clone(), Default::default());
                *guard = Some(RendererInner { engine, renderer });
            }
        }

        // Build the view (borrows self.state temporarily)
        let view = self.state.view();
        let logical_size = iced_core::Size::new(ctx.screen_size.0, ctx.screen_size.1);
        let cache = iced_runtime::user_interface::Cache::new();

        let mut guard = self.inner.lock().unwrap();
        let inner = guard.as_mut().unwrap();
        let renderer = &mut inner.renderer;

        let mut interface = iced_runtime::user_interface::UserInterface::build(
            view,
            logical_size,
            cache,
            renderer,
        );

        let waker = iced_winit::core::shell::Waker::noop();
        let mut bus = iced_winit::core::shell::Bus::new();
        let cursor = iced_winit::core::mouse::Cursor::Unavailable;

        let _ = interface.update(
            &NoopWindow,
            &waker,
            &[],
            cursor,
            renderer,
            &mut bus,
        );

        interface.draw(
            renderer,
            &iced_winit::core::Theme::Dark,
            &iced_winit::core::renderer::Style::default(),
            cursor,
        );

        // Drop interface to release the borrow on self.state
        drop(interface);

        // Process messages
        for message in bus {
            self.state.handle_message(message);
        }

        let physical_size = iced_core::Size::new(
            ctx.screen_size.0 as u32,
            ctx.screen_size.1 as u32,
        );

        let viewport = iced_graphics::Viewport::with_physical_size(
            physical_size,
            iced_winit::core::renderer::Scale {
                window: 1.0,
                application: 1.0,
            },
        );

        inner.renderer.present(None, format, ctx.target_view, &viewport);
    }
}

struct NoopWindow;

impl raw_window_handle::HasWindowHandle for NoopWindow {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'static>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::NotSupported)
    }
}

impl raw_window_handle::HasDisplayHandle for NoopWindow {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'static>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::NotSupported)
    }
}

impl std::fmt::Debug for NoopWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoopWindow").finish()
    }
}
