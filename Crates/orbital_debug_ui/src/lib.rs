//! In-game debug overlay for Orbital: performance timings, an ECS inspector and
//! a log console, drawn with Iced over the game render.
//!
//! Add [`DebugUiModule`] to your app and press its toggle key (F1 by default) to
//! show or hide the panels.
//!
//! ```ignore
//! App::new()
//!     .add_module(DebugUiModule::new())
//!     .liftoff(event_loop, settings)
//! ```
//!
//! The panels are pure functions of the ECS world, so they always reflect live
//! data and there is no snapshot to keep in sync.
//!
//! By default the panels rebuild at [`SMOOTH_REFRESH_INTERVAL`] (~30 Hz) so
//! live values and the performance graph stay readable. Ordinary iced panels
//! use the lazy 4 Hz default; call
//! [`DebugUiModule::with_smooth_refresh`] with `false` to make the overlay
//! that cheap as well.

pub mod editable;
pub mod panels;
pub mod sparkline;
pub mod state;

use std::any::TypeId;

use iced_wgpu::Renderer as IcedRenderer;
use iced_wgpu::wgpu::{Device, Queue};
use iced_winit::core::Element;
use orbital_app::input::InputButton;
use orbital_app::{CameraControlSettings, LayerRenderer, Module, RenderSettings};
use orbital_camera::ProjectionMode;
use orbital_debug_render::DebugToggleState;
use orbital_ecs::{IntoSystem, Res, ResMut, System, World};
use orbital_ecs_bridge::{
    ActiveCamera, CameraDescriptor, CameraDirty, DebugSelectedEntity, DeviceResource,
    InputSnapshot, PerformanceLogConfig, TimingsHistoryConfig, VolumetricSettings,
};
use orbital_iced::{
    DEFAULT_REFRESH_INTERVAL, IcedLayerRenderer, IcedState, SMOOTH_REFRESH_INTERVAL,
};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::panels::{console_panel, ecs_panel, edit, performance_panel, rendering_panel};
use crate::state::{DebugMessage, DebugUiState, PanelId};

/// Shows or hides the whole overlay when the configured key is pressed.
///
/// Mirrors `orbital_debug_render`'s toggle so the two overlays behave alike.
/// The edge-detection state lives in [`DebugUiState`] because the ECS has no
/// `IntoSystem` impl taking two `ResMut` parameters.
pub fn sys_debug_ui_toggle(input: Res<InputSnapshot>, mut ui: ResMut<DebugUiState>) {
    let pressed = input
        .0
        .button_state_any(&InputButton::Keyboard(PhysicalKey::Code(ui.toggle_key)))
        .map(|(_, pressed)| pressed)
        .unwrap_or(false);

    if pressed && !ui.toggle_was_pressed() {
        ui.toggle_visibility();
        orbital_core::logging::debug!("debug UI {}", if ui.visible { "shown" } else { "hidden" });
    }

    ui.set_toggle_was_pressed(pressed);
}

/// Keeps the frame-timing history config in sync with the Performance panel.
///
/// Only the visible Performance window needs graph/table history; when it is
/// hidden, `FrameTimings` clears the retained samples.
pub fn sys_sync_timings_history(
    state: Res<DebugUiState>,
    mut config: ResMut<TimingsHistoryConfig>,
) {
    config.enabled = state.is_panel_visible(PanelId::Performance);
    config.window_secs = state.graph_window_secs as f64;
}

/// Registers the debug panels with the runtime.
///
/// Debug-only: in a release build the module does nothing, so a shipped game
/// pays neither the build cost nor the per-frame panel cost.
pub struct DebugUiModule {
    toggle_key: KeyCode,
    /// Keeps the panels available in a release build instead of compiling them
    /// out. Off by default; this is a debug tool.
    allow_in_release: bool,
    /// Rebuild the panels at the smooth cadence instead of the lazy default.
    /// On by default, because the overlay is only shown on demand.
    smooth: bool,
    /// Register the on-screen touch toggle on mobile. It is skipped when the
    /// device cannot provide the shader capability iced requires.
    mobile_toggle: bool,
}

impl Default for DebugUiModule {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugUiModule {
    pub fn new() -> Self {
        Self {
            // F1: F3 and F4 are already used by `orbital_debug_render`.
            toggle_key: KeyCode::F1,
            allow_in_release: false,
            smooth: true,
            mobile_toggle: true,
        }
    }

    pub fn with_toggle_key(mut self, key: KeyCode) -> Self {
        self.toggle_key = key;
        self
    }

    /// Rebuild the panels at ~30 Hz instead of the shared 4 Hz default.
    ///
    /// Smooth refresh is on by default. Turn it off when the overlay's own
    /// rebuild cost would distort the measurements shown in the Performance
    /// panel; interactions still rebuild immediately either way.
    pub fn with_smooth_refresh(mut self, smooth: bool) -> Self {
        self.smooth = smooth;
        self
    }

    /// Keep the panels available in a release build instead of compiling them
    /// out. Off by default.
    pub fn allow_in_release(mut self, allow: bool) -> Self {
        self.allow_in_release = allow;
        self
    }

    /// Enable or disable the on-screen mobile debug toggle.
    ///
    /// It is skipped automatically on devices without the shader capability
    /// iced needs. Use this to hide it explicitly on platforms with a keyboard.
    pub fn with_mobile_toggle(mut self, enabled: bool) -> Self {
        self.mobile_toggle = enabled;
        self
    }

    fn is_enabled(&self) -> bool {
        self.allow_in_release || cfg!(debug_assertions)
    }

    /// How often the debug panels may rebuild their widget trees.
    fn refresh_interval(&self) -> std::time::Duration {
        if self.smooth {
            SMOOTH_REFRESH_INTERVAL
        } else {
            DEFAULT_REFRESH_INTERVAL
        }
    }
}

impl Module for DebugUiModule {
    fn setup(&self, ecs: &mut World, _device: &Device, _queue: &Queue) -> Vec<Box<dyn System>> {
        if !self.is_enabled() {
            return vec![];
        }

        ecs.insert_resource(DebugUiState::new(self.toggle_key));
        ecs.insert_resource(DebugSelectedEntity::default());

        if ecs.get_resource::<TimingsHistoryConfig>().is_none() {
            ecs.insert_resource(TimingsHistoryConfig::default());
        }

        vec![
            sys_debug_ui_toggle.into_system(),
            sys_sync_timings_history.into_system(),
        ]
    }

    fn register_overlays(
        &self,
        ecs: &mut World,
        layer_renderers: &mut Vec<Box<dyn LayerRenderer>>,
    ) {
        if !self.is_enabled() {
            return;
        }

        if ecs.get_resource::<DebugUiState>().is_none() {
            orbital_core::logging::warn!(
                "DebugUiModule: DebugUiState missing; register_overlays ran before setup"
            );
            return;
        }

        crate::editable::register_builtin_editable_fields();

        // Each panel is an `IcedLayerRenderer<DebugMessage>` pushed straight
        // into `layer_renderers` rather than going through `IcedUiState`. That
        // keeps these messages out of the application's own panel messages, and
        // avoids the module-ordering constraint `IcedUiState` implies.
        //
        // The toggle lives in the ECS world, so visibility is supplied as a
        // predicate evaluated against the world when the view is built. That
        // hides the whole panel — title bar included — instead of blanking only
        // its content, which used to leave the titles on screen while the
        // overlay was off.
        //
        // Tall list panels get an explicit size so they stay usable next to
        // each other; the windows are draggable and resizable, so this is only
        // a starting point. The console is deliberately wide and short, since
        // log lines are long.
        let refresh_interval = self.refresh_interval();
        let panels: [PanelSpec; 4] = [
            (
                "Performance",
                (40.0, 40.0),
                (440.0, 380.0),
                PanelId::Performance,
                performance_panel as PanelView,
            ),
            (
                "ECS",
                (40.0, 420.0),
                (600.0, 460.0),
                PanelId::Ecs,
                ecs_panel as PanelView,
            ),
            (
                "Console",
                (500.0, 40.0),
                (760.0, 260.0),
                PanelId::Console,
                console_panel as PanelView,
            ),
            (
                "Rendering",
                (960.0, 320.0),
                (300.0, 360.0),
                PanelId::Rendering,
                rendering_panel as PanelView,
            ),
        ];

        for (title, position, size, id, view) in panels {
            let panel = IcedState::<DebugMessage>::titled(title)
                .with_position(position.0, position.1)
                .with_size(size.0, size.1)
                .with_close_message(DebugMessage::HidePanel(id))
                // `handle_message` is the only place holding both the world and
                // the panel state, so it is where a component edit is written.
                .with_message_handler(handle_debug_message)
                // Rebuilding the tree means re-running the widget diff and
                // layout, and these panels re-read the whole world to do it.
                // The overlay is shown on demand, so it defaults to the smooth
                // cadence; `with_smooth_refresh(false)` restores the lazy one.
                // Interactions still rebuild immediately either way:
                // `handle_message` marks the panel dirty.
                .with_refresh_interval(refresh_interval)
                .with_visibility(move |world: &World| {
                    world
                        .get_resource::<DebugUiState>()
                        .is_some_and(|state| state.is_panel_visible(id))
                })
                .with_view(view);

            layer_renderers.push(Box::new(IcedLayerRenderer::new(panel)));
        }

        if self.mobile_toggle && cfg!(any(target_os = "android", target_os = "ios")) {
            register_mobile_toggle(layer_renderers);
        }

        orbital_core::logging::debug!("DebugUiModule: registered debug panels");
    }
}

/// Registers the on-screen touch toggle used on platforms without a keyboard.
///
/// Kept as a normal function rather than inline under `cfg` so the desktop
/// build type-checks the same registration path.
#[allow(dead_code)]
fn register_mobile_toggle(layer_renderers: &mut Vec<Box<dyn LayerRenderer>>) {
    let mobile_toggle = IcedState::<DebugMessage>::new()
        .with_hud()
        .with_message_handler(handle_debug_message)
        .with_visibility(mobile_debug_toggle_visible)
        .with_view(mobile_debug_toggle);

    layer_renderers.push(Box::new(IcedLayerRenderer::new(mobile_toggle)));
}

/// Shows the mobile touch toggle only when the overlay or every panel is
/// hidden, so it does not cover the live panels while they are open.
#[allow(dead_code)]
fn mobile_debug_toggle_visible(world: &World) -> bool {
    let supports_f16 = world
        .get_resource::<DeviceResource>()
        .is_some_and(|device| {
            device
                .0
                .features()
                .contains(iced_wgpu::wgpu::Features::SHADER_F16)
        });
    if !supports_f16 {
        return false;
    }

    world
        .get_resource::<DebugUiState>()
        .is_some_and(|state| !state.visible || state.panels.iter().all(|visible| !visible))
}

/// A small always-available touch target that opens the debug overlay.
///
/// Desktop and laptop users have the configured toggle key; phones and tablets
/// usually have no keyboard, so they need an on-screen way back into the
/// panels after the overlay is hidden or every window was closed.
#[allow(dead_code)]
fn mobile_debug_toggle(
    world: &World,
) -> Element<'static, DebugMessage, iced_winit::core::Theme, IcedRenderer> {
    if !mobile_debug_toggle_visible(world) {
        return crate::panels::blank();
    }

    let button = iced_widget::button(iced_widget::text("Debug").size(18))
        .on_press(DebugMessage::ShowDebugUi)
        .padding([10, 18]);

    iced_widget::container(button).padding(16).into()
}

/// Applies a debug panel's message to the world.
///
/// Everything a panel changes lives in the `DebugUiState` resource, so this
/// routes the message there and performs any component write.
fn handle_debug_message(
    _panel: &mut IcedState<DebugMessage>,
    message: DebugMessage,
    world: &mut World,
) {
    if let DebugMessage::ApplyField(ref key) = message {
        let input = world
            .get_resource::<DebugUiState>()
            .and_then(|state| state.field_buffers.get(key).cloned())
            .unwrap_or_default();

        let result = edit::apply_edit(world, key, &input);
        let failure = result.as_ref().err().cloned();

        if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
            state.last_error = failure;
            // Drop the buffer so the field falls back to the component's value.
            state.field_buffers.remove(key);
            state.dirty = true;
        }

        match result {
            Ok(()) => orbital_core::logging::debug!(
                "debug UI: applied field on entity {}",
                key.entity.index
            ),
            Err(error) => orbital_core::logging::warn!("debug UI edit failed: {error}"),
        }

        return;
    }

    if let DebugMessage::SelectEntity(entity) = &message
        && let Some(mut selected) = world.get_resource_mut::<DebugSelectedEntity>()
    {
        selected.0 = *entity;
    }

    match message {
        DebugMessage::SetNumericField(key, value) => {
            let input = format!("{value}");
            let result = edit::apply_edit(world, &key, &input);
            if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
                if result.is_ok() {
                    // Clear the text buffer so the field falls back to the
                    // slider's current value.
                    state.field_buffers.remove(&key);
                    state.last_error = None;
                } else {
                    state.last_error = result.err();
                }
                state.dirty = true;
            }
            return;
        }
        DebugMessage::AddModelInstance(entity) => {
            edit::add_model_instance(world, entity);
            if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
                state.dirty = true;
            }
            return;
        }
        DebugMessage::RemoveModelInstance { entity, ulid } => {
            edit::remove_model_instance(world, entity, ulid);
            if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
                state.dirty = true;
            }
            return;
        }
        DebugMessage::DespawnEntity(entity) => {
            let result = edit::despawn_entity(world, entity);
            if let Some(mut selected) = world.get_resource_mut::<DebugSelectedEntity>()
                && selected.0 == Some(entity)
            {
                selected.0 = None;
            }
            if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
                match &result {
                    Ok(()) => {
                        if state.selected == Some(entity) {
                            state.selected = None;
                        }
                        state.expanded.remove(&entity);
                        state.field_buffers.retain(|key, _| key.entity != entity);
                        state.last_error = None;
                    }
                    Err(error) => {
                        state.last_error = Some(error.clone());
                    }
                }
                state.dirty = true;
            }
            return;
        }
        DebugMessage::SetAntiAliasing(mode) => {
            let mut settings = world
                .get_resource::<RenderSettings>()
                .map(|settings| *settings)
                .unwrap_or_default();
            settings.anti_aliasing = mode;
            world.insert_resource(settings);
            mark_debug_dirty(world);
            return;
        }
        DebugMessage::SetSmaaQuality(quality) => {
            let mut settings = world
                .get_resource::<RenderSettings>()
                .map(|settings| *settings)
                .unwrap_or_default();
            settings.smaa_quality = quality;
            world.insert_resource(settings);
            mark_debug_dirty(world);
            return;
        }
        DebugMessage::SetRenderScale(scale) => {
            update_render_settings(world, |settings| {
                settings.render_scale =
                    scale.clamp(settings.min_render_scale, settings.max_render_scale);
                settings.adaptive_render_scale = false;
            });
            return;
        }
        DebugMessage::SetRenderScalePercent(percent) => {
            update_render_settings(world, |settings| {
                settings.render_scale = (percent as f32 / 100.0)
                    .clamp(settings.min_render_scale, settings.max_render_scale);
                settings.adaptive_render_scale = false;
            });
            return;
        }
        DebugMessage::SetAdaptiveRenderScale(enabled) => {
            update_render_settings(world, |settings| settings.adaptive_render_scale = enabled);
            return;
        }
        DebugMessage::SetPerformanceTargetFps(fps) => {
            update_render_settings(world, |settings| settings.target_fps = fps.clamp(1, 1000));
            return;
        }
        DebugMessage::SetFrameCapEnabled(enabled) => {
            update_render_settings(world, |settings| {
                settings.fps_limit = if enabled {
                    Some(settings.fps_limit.unwrap_or(settings.target_fps.max(1)))
                } else {
                    None
                };
            });
            return;
        }
        DebugMessage::SetFrameCapFps(fps) => {
            update_render_settings(world, |settings| {
                settings.fps_limit = Some(fps.clamp(1, 1000))
            });
            return;
        }
        DebugMessage::SetExposure(exposure) => {
            update_render_settings(world, |settings| settings.exposure = exposure.max(0.0));
            return;
        }
        DebugMessage::SetBloomEnabled(enabled) => {
            update_render_settings(world, |settings| settings.bloom_enabled = enabled);
            return;
        }
        DebugMessage::SetDeferredEnabled(enabled) => {
            update_render_settings(world, |settings| settings.deferred_enabled = enabled);
            return;
        }
        DebugMessage::SetPerformanceLogging(enabled) => {
            let mut config = world
                .get_resource::<PerformanceLogConfig>()
                .map(|config| *config)
                .unwrap_or_default();
            config.console = enabled;
            world.insert_resource(config);
            mark_debug_dirty(world);
            return;
        }
        DebugMessage::SetPerformanceCsv(enabled) => {
            let mut config = world
                .get_resource::<PerformanceLogConfig>()
                .map(|config| *config)
                .unwrap_or_default();
            config.csv = enabled;
            world.insert_resource(config);
            mark_debug_dirty(world);
            return;
        }
        DebugMessage::SetDebugOverlay(enabled) => {
            update_debug_settings(world, |settings| settings.enabled = enabled);
            return;
        }
        DebugMessage::SetDebugBounds(bounds) => {
            update_debug_settings(world, |settings| settings.bounds = bounds);
            return;
        }
        DebugMessage::SetDebugFrustum(enabled) => {
            update_debug_settings(world, |settings| settings.frustum = enabled);
            return;
        }
        DebugMessage::SetDebugLights(enabled) => {
            update_debug_settings(world, |settings| settings.lights = enabled);
            return;
        }
        DebugMessage::SetDebugWireframe(enabled) => {
            update_debug_settings(world, |settings| settings.wireframe = enabled);
            return;
        }
        DebugMessage::SetDebugGridAxes(enabled) => {
            update_debug_settings(world, |settings| settings.grid_axes = enabled);
            return;
        }
        DebugMessage::SetDebugShadowFrusta(enabled) => {
            update_debug_settings(world, |settings| settings.shadow_frusta = enabled);
            return;
        }
        DebugMessage::SetDebugSelection(enabled) => {
            update_debug_settings(world, |settings| settings.selection = enabled);
            return;
        }
        DebugMessage::SetBloomThreshold(threshold) => {
            update_render_settings(world, |settings| {
                settings.bloom_threshold = threshold.max(0.0)
            });
            return;
        }
        DebugMessage::SetBloomIntensity(intensity) => {
            update_render_settings(world, |settings| {
                settings.bloom_intensity = intensity.max(0.0)
            });
            return;
        }
        DebugMessage::SetContrast(contrast) => {
            update_render_settings(world, |settings| settings.contrast = contrast.max(0.0));
            return;
        }
        DebugMessage::SetLightGlare(enabled) => {
            update_render_settings(world, |settings| settings.light_glare = enabled);
            return;
        }
        DebugMessage::SetGlareStars(enabled) => {
            update_render_settings(world, |settings| settings.glare_stars = enabled);
            return;
        }
        DebugMessage::SetGlareArms(arms) => {
            update_render_settings(world, |settings| {
                settings.glare_star_arms = arms.clamp(2, 8)
            });
            return;
        }
        DebugMessage::SetGlareStarInner(value) => {
            update_render_settings(world, |settings| settings.glare_star_inner = value.max(0.0));
            return;
        }
        DebugMessage::SetGlareStarOuter(value) => {
            update_render_settings(world, |settings| settings.glare_star_outer = value.max(0.0));
            return;
        }
        DebugMessage::SetGlareStarLength(value) => {
            update_render_settings(world, |settings| {
                settings.glare_star_length = value.clamp(1.0, 12.0)
            });
            return;
        }
        DebugMessage::SetProjection(projection) => {
            apply_projection(world, projection);
            mark_debug_dirty(world);
            return;
        }
        DebugMessage::SetOrthoForwardMode(mode) => {
            let mut settings = world
                .get_resource::<CameraControlSettings>()
                .map(|settings| *settings)
                .unwrap_or_default();
            settings.orthographic_forward = mode;
            world.insert_resource(settings);
            mark_debug_dirty(world);
            return;
        }
        DebugMessage::SetVolumetricsEnabled(enabled) => {
            update_volumetrics(world, |settings| settings.enabled = enabled);
            return;
        }
        DebugMessage::SetVolumetricDensity(density) => {
            update_volumetrics(world, |settings| settings.density = density.max(0.0));
            return;
        }
        DebugMessage::SetVolumetricHeightBase(height) => {
            update_volumetrics(world, |settings| settings.height_base = height);
            return;
        }
        DebugMessage::SetVolumetricHeightDensity(density) => {
            update_volumetrics(world, |settings| {
                settings.height_density = density.max(0.0);
            });
            return;
        }
        DebugMessage::SetVolumetricHeightFalloff(falloff) => {
            update_volumetrics(world, |settings| {
                settings.height_falloff = falloff.max(0.0);
            });
            return;
        }
        DebugMessage::SetVolumetricScattering(scattering) => {
            update_volumetrics(world, |settings| {
                settings.scattering = scattering.max(0.0);
            });
            return;
        }
        DebugMessage::SetVolumetricExtinction(extinction) => {
            update_volumetrics(world, |settings| {
                settings.extinction = extinction.max(0.0);
            });
            return;
        }
        DebugMessage::SetVolumetricEnvironmentIntensity(intensity) => {
            update_volumetrics(world, |settings| {
                settings.environment_intensity = intensity.max(0.0);
            });
            return;
        }
        DebugMessage::SetVolumetricAnisotropy(anisotropy) => {
            update_volumetrics(world, |settings| {
                settings.anisotropy = anisotropy.clamp(-0.95, 0.95);
            });
            return;
        }
        DebugMessage::SetVolumetricMaxDistance(distance) => {
            update_volumetrics(world, |settings| {
                settings.max_distance = distance.max(0.1);
            });
            return;
        }
        DebugMessage::SetVolumetricTemporalHistory(enabled) => {
            update_volumetrics(world, |settings| settings.temporal_history = enabled);
            return;
        }
        DebugMessage::SetVolumetricQuality(quality) => {
            update_volumetrics(world, |settings| settings.quality = quality);
            return;
        }
        _ => {}
    }

    if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
        state.apply(&message);
    }
}

/// Marks the debug panels dirty after a world-side control change.
fn mark_debug_dirty(world: &mut World) {
    if let Some(mut state) = world.get_resource_mut::<DebugUiState>() {
        state.dirty = true;
    }
}

/// Mutates the debug-render settings when the debug-render module is present.
fn update_debug_settings(
    world: &mut World,
    update: impl FnOnce(&mut orbital_debug_render::DebugRenderSettings),
) {
    let changed = if let Some(mut toggle) = world.get_resource_mut::<DebugToggleState>() {
        update(&mut toggle.settings);
        true
    } else {
        false
    };

    if changed {
        mark_debug_dirty(world);
    }
}

/// Mutates the render settings, creating defaults if the app had not inserted
/// the resource yet.
fn update_render_settings(world: &mut World, update: impl FnOnce(&mut RenderSettings)) {
    let mut settings = world
        .get_resource::<RenderSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    update(&mut settings);
    world.insert_resource(settings);
    mark_debug_dirty(world);
}

/// Mutates the volumetric settings, creating defaults if the app had not
/// inserted the resource yet.
fn update_volumetrics(world: &mut World, update: impl FnOnce(&mut VolumetricSettings)) {
    let mut settings = world
        .get_resource::<VolumetricSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    update(&mut settings);
    world.insert_resource(settings);
    mark_debug_dirty(world);
}

/// Switches the active camera's projection and marks it for realization.
fn apply_projection(world: &mut World, projection: ProjectionMode) {
    let Some(active) = world.get_resource::<ActiveCamera>().map(|active| active.0) else {
        return;
    };

    let changed = world
        .component_mut_any(TypeId::of::<CameraDescriptor>(), &active)
        .and_then(|mut handle| {
            handle
                .as_any_mut()
                .downcast_mut::<CameraDescriptor>()
                .map(|camera| {
                    if camera.projection_mode == projection {
                        false
                    } else {
                        camera.projection_mode = projection;
                        true
                    }
                })
        })
        .unwrap_or(false);

    if changed {
        let _ = world.attach_component(&active, CameraDirty(true));
        orbital_core::logging::debug!(
            "debug UI: camera {} projection set to {projection}",
            active.index
        );
    }
}

/// A panel view: a pure function from the world to a widget tree.
type PanelView =
    fn(&World) -> Element<'static, DebugMessage, iced_winit::core::Theme, IcedRenderer>;

/// A panel's title, initial position, initial size, identity and view.
type PanelSpec = (
    &'static str,
    (f32, f32),
    (f32, f32),
    crate::state::PanelId,
    PanelView,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_toggle_is_enabled_by_default_and_can_be_opted_out() {
        assert!(DebugUiModule::new().mobile_toggle);
        assert!(!DebugUiModule::new().with_mobile_toggle(false).mobile_toggle);
    }

    #[test]
    fn smooth_refresh_is_the_default() {
        assert_eq!(
            DebugUiModule::new().refresh_interval(),
            SMOOTH_REFRESH_INTERVAL
        );
    }

    #[test]
    fn smooth_refresh_can_be_disabled() {
        assert_eq!(
            DebugUiModule::new()
                .with_smooth_refresh(false)
                .refresh_interval(),
            DEFAULT_REFRESH_INTERVAL
        );
    }

    #[test]
    fn projection_control_updates_the_active_camera() {
        let mut world = World::new();
        let camera = world.spawn_entity();
        world
            .attach_component(&camera, CameraDescriptor::default())
            .expect("camera attach failed");
        world.insert_resource(ActiveCamera(camera));

        apply_projection(&mut world, ProjectionMode::Orthographic);

        let projection = world
            .component_debug(TypeId::of::<CameraDescriptor>(), &camera)
            .and_then(|handle| {
                handle
                    .as_any()
                    .downcast_ref::<CameraDescriptor>()
                    .map(|camera| camera.projection_mode)
            });
        assert_eq!(projection, Some(ProjectionMode::Orthographic));

        let dirty = world
            .component_debug(TypeId::of::<CameraDirty>(), &camera)
            .and_then(|handle| {
                handle
                    .as_any()
                    .downcast_ref::<CameraDirty>()
                    .map(|dirty| dirty.0)
            });
        assert_eq!(dirty, Some(true));
    }

    #[test]
    fn volumetric_control_creates_the_resource_with_defaults() {
        let mut world = World::new();

        update_volumetrics(&mut world, |settings| settings.density = 0.02);

        let density = world
            .get_resource::<VolumetricSettings>()
            .map(|settings| settings.density);
        assert_eq!(density, Some(0.02));
    }
}
