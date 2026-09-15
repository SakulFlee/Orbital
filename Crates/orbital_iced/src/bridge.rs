use crate::renderer::IcedLayerRenderer;
use crate::state::IcedUiState;
use orbital_app::render_overlay::RenderOverlayResource;
use orbital_app::Module;
use orbital_ecs::{System, World};
use wgpu::{Device, Queue};

/// Engine module that bridges ECS-declared iced UI state to the render pipeline.
///
/// Add this module to your [`App`](orbital_app::App) after any modules that
/// insert [`IcedUiState`]. The bridge will detect the state and automatically
/// create an [`IcedLayerRenderer`] for it.
pub struct IcedBridgeModule;

impl Module for IcedBridgeModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &Device,
        _queue: &Queue,
    ) -> Vec<Box<dyn System>> {
        let ui_state = ecs.get_resource::<IcedUiState>().map(|s| s.0.clone());

        if let Some(state) = ui_state {
            let renderer = IcedLayerRenderer::new(state);

            if ecs.get_resource::<RenderOverlayResource>().is_none() {
                ecs.insert_resource(RenderOverlayResource::new());
            }
            if let Some(res) = ecs.get_resource_mut::<RenderOverlayResource>() {
                res.add_layer_renderer(Box::new(renderer));
            }

            log::info!("Iced UI bridge: registered renderer");
        }

        vec![]
    }
}
