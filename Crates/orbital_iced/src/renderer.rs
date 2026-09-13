use crate::state::IcedState;
use orbital_app::render_overlay::{RenderOverlay, RenderOverlayContext};
use orbital_app::RenderLayer;

pub struct IcedLayerRenderer {
    state: IcedState,
}

impl IcedLayerRenderer {
    pub fn new(state: IcedState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &IcedState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut IcedState {
        &mut self.state
    }
}

impl RenderOverlay for IcedLayerRenderer {
    fn render(&mut self, _ctx: RenderOverlayContext) {
        // TODO: Initialize iced renderer with ctx.device, ctx.queue, ctx.target_view
        // TODO: Run iced UI logic here
    }
}

pub struct IcedLayerOverlay {
    state: IcedState,
}

impl IcedLayerOverlay {
    pub fn new(state: IcedState) -> Self {
        Self { state }
    }
}

impl orbital_app::render_overlay::LayerRenderer for IcedLayerOverlay {
    fn layer(&self) -> RenderLayer {
        RenderLayer::UI
    }

    fn render(&mut self, _ctx: RenderOverlayContext) {
        // TODO: Initialize iced renderer and draw UI at UI layer
    }
}
