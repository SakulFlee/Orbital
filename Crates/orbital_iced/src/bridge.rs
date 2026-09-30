use crate::renderer::IcedLayerRenderer;
use crate::state::{IcedState, IcedUiState, Message};
use orbital_app::Module;
use orbital_app::render_overlay::{LayerRenderer, RenderOverlay};
use orbital_ecs::{System, World};
use wgpu::{Device, Queue};

/// Engine module that bridges ECS-declared iced UI state to the render pipeline.
///
/// Add this module to your [`App`](orbital_app::App) after any modules that
/// insert [`IcedUiState`]. The bridge will detect the state and automatically
/// create an [`IcedLayerRenderer`] for each panel.
pub struct IcedBridgeModule;

impl Module for IcedBridgeModule {
    fn setup(&self, _ecs: &mut World, _device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        // Panels are read during register_overlays(), not here.
        vec![]
    }

    fn register_overlays(
        &self,
        ecs: &mut World,
        layer_renderers: &mut Vec<Box<dyn LayerRenderer>>,
        _legacy_overlays: &mut Vec<Box<dyn RenderOverlay>>,
    ) {
        let panels = ecs
            .get_resource::<IcedUiState>()
            .map(|ui| {
                ui.0.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if panels.is_empty() {
            return;
        }

        for (name, state) in panels {
            let renderer = IcedLayerRenderer::new(apply_builtin_defaults(state));
            layer_renderers.push(Box::new(renderer));
            log::info!("Iced UI bridge: registered panel '{}'", name);
        }
    }
}

/// Fills in the behaviour panels used to get implicitly, now that
/// `IcedState` is generic over its message type.
///
/// A panel declared through [`IcedUiState`] that does not supply a view gets
/// the built-in button, and a windowed panel that does not name a close message
/// gets [`Message::ClosePanel`]. Without this, generalising the close button
/// would silently drop the close button from every existing panel.
pub(crate) fn apply_builtin_defaults(mut state: IcedState<Message>) -> IcedState<Message> {
    if !state.has_view() {
        state = state.with_builtin_view();
    }

    if !state.has_close_message() {
        state.set_close_message(Message::ClosePanel);
    }

    state
}
