use std::sync::Mutex;

use egui::Context;
use orbital_app::{Module, RenderOverlayResource};
use orbital_ecs::{IntoSystem, System, World};
use wgpu::{Device, Queue, TextureFormat};
use winit::keyboard::KeyCode;

use crate::overlay::EguiOverlay;
use crate::state::{EguiPanels, EguiState};
use crate::system::sys_egui_toggle;
use crate::ui;

/// Module that adds an egui debug UI overlay to the application.
///
/// # Example
///
/// ```ignore
/// App::new()
///     .add_module(
///         EguiModule::new()
///             .with_toggle_key(KeyCode::F2)
///             .with_panel(ui::performance::PerformancePanel),
///     )
///     .liftoff(event_loop, settings);
/// ```
pub struct EguiModule {
    toggle_key: KeyCode,
    panels: Mutex<Vec<Box<dyn ui::Panel>>>,
}

impl EguiModule {
    pub fn new() -> Self {
        Self {
            toggle_key: KeyCode::F2,
            panels: Mutex::new(Vec::new()),
        }
    }

    /// Set the key that toggles the egui overlay. Defaults to F2.
    pub fn with_toggle_key(mut self, key: KeyCode) -> Self {
        self.toggle_key = key;
        self
    }

    /// Add a panel to the egui overlay.
    pub fn with_panel(self, panel: impl ui::Panel + 'static) -> Self {
        self.panels.lock().unwrap().push(Box::new(panel));
        self
    }
}

impl Default for EguiModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for EguiModule {
    fn setup(&self, ecs: &mut World, device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        let format = ecs
            .get_resource::<orbital_ecs_bridge::SurfaceFormatResource>()
            .map(|f| f.0)
            .unwrap_or(TextureFormat::Bgra8UnormSrgb);

        let ctx = Context::default();

        // egui_winit::State requires a &dyn HasDisplayHandle (Window), which we
        // don't have during Module::setup(). The overlay defers State creation
        // to the first frame via the `initialized` flag in on_window_event/render.
        // We store None here and create it when we have access to the window.

        let wgpu_renderer = egui_wgpu::Renderer::new(
            device,
            format,
            egui_wgpu::RendererOptions {
                depth_stencil_format: None,
                ..Default::default()
            },
        );

        // Insert ECS resources
        ecs.insert_resource(EguiState {
            ctx: ctx.clone(),
            enabled: true,
            toggle_key: self.toggle_key,
            was_pressed: false,
        });

        // Move panels from the module into the ECS resource.
        // Any module can add panels later via get_resource_mut::<EguiPanels>().
        let panels = std::mem::take(&mut *self.panels.lock().unwrap());
        ecs.insert_resource(EguiPanels(panels));

        // Create the overlay (no panels stored on it — reads from ECS)
        let overlay = EguiOverlay {
            egui_ctx: ctx,
            winit_state: None, // deferred to first frame
            wgpu_renderer,
            initialized: false,
        };

        ecs.insert_resource(RenderOverlayResource(Mutex::new(Box::new(overlay))));

        vec![sys_egui_toggle.into_system()]
    }
}
