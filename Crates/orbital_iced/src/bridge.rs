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
/// create an [`IcedLayerRenderer`] for each panel.
pub struct IcedBridgeModule;

impl Module for IcedBridgeModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &Device,
        _queue: &Queue,
    ) -> Vec<Box<dyn System>> {
        let panels = ecs
            .get_resource::<IcedUiState>()
            .map(|ui| ui.0.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>())
            .unwrap_or_default();

        if panels.is_empty() {
            return vec![];
        }

        if ecs.get_resource::<RenderOverlayResource>().is_none() {
            ecs.insert_resource(RenderOverlayResource::new());
        }

        for (name, state) in panels {
            let renderer = IcedLayerRenderer::new(state);
            if let Some(res) = ecs.get_resource_mut::<RenderOverlayResource>() {
                res.add_layer_renderer(Box::new(renderer));
            }
            log::info!("Iced UI bridge: registered panel '{}'", name);
        }

        vec![]
    }
}
