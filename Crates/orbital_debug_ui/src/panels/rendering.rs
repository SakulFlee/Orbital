//! Panel exposing live renderer and volumetrics controls.

use std::any::TypeId;

use iced_widget::{checkbox, column, container, pick_list, radio, row, slider, text};
use iced_winit::core::{Color, Element, Length, Theme};

use orbital_app::{CameraControlSettings, OrthographicForwardMode, RenderSettings};
use orbital_camera::ProjectionMode;
use orbital_debug_render::{BoundsMode, DebugToggleState};
use orbital_ecs::World;
use orbital_ecs_bridge::{
    ActiveCamera, CameraDescriptor, LightDescriptor, ModelDescriptor, VolumetricQuality,
    VolumetricSettings, WindowSize,
};
use orbital_material_shader::RenderQueue;
use orbital_renderer::{AntiAliasing, SmaaQuality};

use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState};

const LABEL_WIDTH: f32 = 108.0;
const VALUE_WIDTH: f32 = 46.0;
const ACCENT: Color = Color::from_rgb(0.7, 0.8, 1.0);
const MUTED: Color = Color::from_rgb(0.6, 0.6, 0.66);

/// Builds the rendering controls panel.
pub fn rendering_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    if ecs.get_resource::<DebugUiState>().is_none() {
        return crate::panels::blank();
    }

    let render = ecs
        .get_resource::<RenderSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    let camera_controls = ecs
        .get_resource::<CameraControlSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    let volumetrics = ecs
        .get_resource::<VolumetricSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    let projection = active_projection(ecs);

    let mut rows: Vec<Element<'static, DebugMessage, Theme, IcedRenderer>> = Vec::new();

    rows.push(crate::panels::caption("Performance target", ACCENT));
    rows.push(
        checkbox(render.adaptive_render_scale)
            .label("Adaptive render scale")
            .text_size(12)
            .on_toggle(DebugMessage::SetAdaptiveRenderScale)
            .into(),
    );
    if render.adaptive_render_scale {
        rows.push(
            column![
                row![
                    text("Target FPS")
                        .size(12)
                        .width(Length::Fixed(LABEL_WIDTH)),
                    text(render.target_fps.to_string())
                        .size(11)
                        .color(MUTED)
                        .width(Length::Fixed(VALUE_WIDTH)),
                ],
                slider(30.0..=144.0, render.target_fps as f32, |value| {
                    DebugMessage::SetPerformanceTargetFps(value.round() as u32)
                })
                .step(1.0),
            ]
            .spacing(1)
            .into(),
        );
    } else {
        rows.push(
            row![
                radio(
                    "1x",
                    100u32,
                    Some((render.render_scale * 100.0).round() as u32),
                    DebugMessage::SetRenderScalePercent,
                )
                .text_size(12),
                radio(
                    "0.75x",
                    75u32,
                    Some((render.render_scale * 100.0).round() as u32),
                    DebugMessage::SetRenderScalePercent,
                )
                .text_size(12),
                radio(
                    "0.5x",
                    50u32,
                    Some((render.render_scale * 100.0).round() as u32),
                    DebugMessage::SetRenderScalePercent,
                )
                .text_size(12),
                radio(
                    "0.25x",
                    25u32,
                    Some((render.render_scale * 100.0).round() as u32),
                    DebugMessage::SetRenderScalePercent,
                )
                .text_size(12),
            ]
            .spacing(8)
            .into(),
        );
        rows.push(slider_row(
            "Render scale",
            render.min_render_scale..=render.max_render_scale,
            render.render_scale,
            DebugMessage::SetRenderScale,
        ));
    }

    if let Some(surface) = ecs.get_resource::<WindowSize>() {
        let render_width = (surface.0.x as f32 * render.render_scale).round() as u32;
        let render_height = (surface.0.y as f32 * render.render_scale).round() as u32;
        rows.push(
            text(format!(
                "Surface {}x{} · render {}x{} · scale {:.2}x",
                surface.0.x, surface.0.y, render_width, render_height, render.render_scale,
            ))
            .size(10)
            .color(MUTED)
            .into(),
        );
    }
    rows.push(
        text("Only the 3D scene is scaled; 2D, UI, and text stay native.")
            .size(10)
            .color(MUTED)
            .into(),
    );

    rows.push(crate::panels::divider());
    rows.push(crate::panels::caption("Frame pacing", ACCENT));
    rows.push(
        checkbox(render.fps_limit.is_some())
            .label("Frame cap")
            .text_size(12)
            .on_toggle(DebugMessage::SetFrameCapEnabled)
            .into(),
    );
    if let Some(limit) = render.fps_limit {
        rows.push(
            column![
                row![
                    text("Cap FPS").size(12).width(Length::Fixed(LABEL_WIDTH)),
                    text(limit.to_string())
                        .size(11)
                        .color(MUTED)
                        .width(Length::Fixed(VALUE_WIDTH)),
                ],
                slider(15.0..=240.0, limit as f32, |value| {
                    DebugMessage::SetFrameCapFps(value.round() as u32)
                })
                .step(1.0),
            ]
            .spacing(1)
            .into(),
        );
    }
    rows.push(
        text("Ignored while VSync is enabled.")
            .size(10)
            .color(MUTED)
            .into(),
    );

    rows.push(crate::panels::divider());
    rows.push(crate::panels::caption("Anti-aliasing", ACCENT));
    rows.push(
        row![
            radio(
                "Off",
                AntiAliasing::Off,
                Some(render.anti_aliasing),
                DebugMessage::SetAntiAliasing,
            ),
            radio(
                "FXAA",
                AntiAliasing::Fxaa,
                Some(render.anti_aliasing),
                DebugMessage::SetAntiAliasing,
            ),
            radio(
                "SMAA",
                AntiAliasing::Smaa,
                Some(render.anti_aliasing),
                DebugMessage::SetAntiAliasing,
            ),
        ]
        .spacing(8)
        .into(),
    );

    if render.anti_aliasing == AntiAliasing::Smaa {
        rows.push(crate::panels::caption("SMAA quality", ACCENT));
        rows.push(
            row![
                radio(
                    "Low",
                    SmaaQuality::Low,
                    Some(render.smaa_quality),
                    DebugMessage::SetSmaaQuality,
                )
                .text_size(12),
                radio(
                    "Medium",
                    SmaaQuality::Medium,
                    Some(render.smaa_quality),
                    DebugMessage::SetSmaaQuality,
                )
                .text_size(12),
                radio(
                    "High",
                    SmaaQuality::High,
                    Some(render.smaa_quality),
                    DebugMessage::SetSmaaQuality,
                )
                .text_size(12),
                radio(
                    "Ultra",
                    SmaaQuality::Ultra,
                    Some(render.smaa_quality),
                    DebugMessage::SetSmaaQuality,
                )
                .text_size(12),
            ]
            .spacing(6)
            .into(),
        );
    }

    rows.push(crate::panels::divider());
    rows.push(crate::panels::caption("Rendering path", ACCENT));
    let model_count = ecs
        .get_component_store::<ModelDescriptor>()
        .map(|store| store.dense.len())
        .unwrap_or(0);
    let light_count = ecs
        .get_component_store::<LightDescriptor>()
        .map(|store| store.dense.len())
        .unwrap_or(0);
    let (opaque_draws, transparent_draws) = ecs
        .get_component_store::<ModelDescriptor>()
        .map(|models| {
            models
                .components
                .iter()
                .flat_map(|model| model.materials.iter())
                .fold(
                    (0usize, 0usize),
                    |(opaque, transparent), material| match material.render_queue {
                        RenderQueue::Opaque => (opaque + 1, transparent),
                        RenderQueue::Transparent => (opaque, transparent + 1),
                    },
                )
        })
        .unwrap_or((0, 0));
    rows.push(
        text(format!(
            "{model_count} models · {light_count} lights · {opaque_draws} opaque / {transparent_draws} transparent draws"
        ))
        .size(10)
        .color(MUTED)
        .into(),
    );
    rows.push(
        checkbox(render.deferred_enabled)
            .label("Deferred (hybrid)")
            .text_size(12)
            .on_toggle(DebugMessage::SetDeferredEnabled)
            .into(),
    );
    rows.push(
        text("Disable to force opaque PBR materials through forward rendering.")
            .size(10)
            .color(MUTED)
            .into(),
    );

    rows.push(crate::panels::divider());
    rows.push(crate::panels::caption("Exposure & bloom", ACCENT));
    rows.push(slider_row(
        "Exposure",
        0.1..=4.0,
        render.exposure,
        DebugMessage::SetExposure,
    ));
    rows.push(slider_row(
        "Contrast",
        0.5..=2.0,
        render.contrast,
        DebugMessage::SetContrast,
    ));
    rows.push(
        checkbox(render.bloom_enabled)
            .label("Bloom / glare")
            .text_size(12)
            .on_toggle(DebugMessage::SetBloomEnabled)
            .into(),
    );
    rows.push(slider_row(
        "Scene threshold",
        0.0..=8.0,
        render.bloom_threshold,
        DebugMessage::SetBloomThreshold,
    ));
    rows.push(slider_row(
        "Intensity",
        0.0..=2.0,
        render.bloom_intensity,
        DebugMessage::SetBloomIntensity,
    ));
    rows.push(
        checkbox(render.light_glare)
            .label("Light glare")
            .text_size(12)
            .on_toggle(DebugMessage::SetLightGlare)
            .into(),
    );
    if render.light_glare {
        rows.push(
            checkbox(render.glare_stars)
                .label("Star flare")
                .text_size(12)
                .on_toggle(DebugMessage::SetGlareStars)
                .into(),
        );
        if render.glare_stars {
            rows.push(
                row![
                    radio(
                        "4",
                        4u32,
                        Some(render.glare_star_arms),
                        DebugMessage::SetGlareArms,
                    )
                    .text_size(12),
                    radio(
                        "6",
                        6u32,
                        Some(render.glare_star_arms),
                        DebugMessage::SetGlareArms,
                    )
                    .text_size(12),
                    radio(
                        "8",
                        8u32,
                        Some(render.glare_star_arms),
                        DebugMessage::SetGlareArms,
                    )
                    .text_size(12),
                ]
                .spacing(8)
                .into(),
            );
            rows.push(slider_row(
                "Inner",
                0.0..=0.5,
                render.glare_star_inner,
                DebugMessage::SetGlareStarInner,
            ));
            rows.push(slider_row(
                "Outer",
                0.0..=0.5,
                render.glare_star_outer,
                DebugMessage::SetGlareStarOuter,
            ));
            rows.push(slider_row(
                "Length",
                1.0..=12.0,
                render.glare_star_length,
                DebugMessage::SetGlareStarLength,
            ));
        }
    }
    rows.push(
        text("Scene highlights above the threshold bloom; light glare uses Intensity.")
            .size(10)
            .color(MUTED)
            .into(),
    );

    rows.push(crate::panels::divider());
    rows.push(crate::panels::caption("Projection", ACCENT));

    let picker = pick_list(
        Some(projection.unwrap_or_default()),
        [ProjectionMode::Perspective, ProjectionMode::Orthographic],
        |mode| mode.to_string(),
    )
    .width(Length::Fill)
    .text_size(12);
    let picker = if projection.is_some() {
        picker.on_select(DebugMessage::SetProjection)
    } else {
        picker
    };
    rows.push(picker.into());
    if projection.is_none() {
        rows.push(text("no active camera").size(11).color(MUTED).into());
    }

    if projection == Some(ProjectionMode::Orthographic) {
        rows.push(crate::panels::divider());
        rows.push(crate::panels::caption("Ortho W/S", ACCENT));
        rows.push(
            row![
                radio(
                    "Dolly",
                    OrthographicForwardMode::Dolly,
                    Some(camera_controls.orthographic_forward),
                    DebugMessage::SetOrthoForwardMode,
                )
                .text_size(12),
                radio(
                    "Depth",
                    OrthographicForwardMode::Depth,
                    Some(camera_controls.orthographic_forward),
                    DebugMessage::SetOrthoForwardMode,
                )
                .text_size(12),
            ]
            .spacing(6)
            .into(),
        );
        rows.push(
            text("Dolly turns W/S into orthographic zoom; Depth keeps view-axis movement.")
                .size(10)
                .color(MUTED)
                .into(),
        );
    }

    rows.push(crate::panels::divider());
    rows.push(crate::panels::caption("Volumetrics", ACCENT));
    rows.push(
        checkbox(volumetrics.enabled)
            .label("Enabled")
            .text_size(12)
            .on_toggle(DebugMessage::SetVolumetricsEnabled)
            .into(),
    );
    rows.push(
        checkbox(volumetrics.temporal_history)
            .label("Temporal history")
            .text_size(12)
            .on_toggle(DebugMessage::SetVolumetricTemporalHistory)
            .into(),
    );

    rows.push(
        text("Base density: uniform fog.")
            .size(10)
            .color(MUTED)
            .into(),
    );
    rows.push(
        text("Height fog: extra density at/under")
            .size(10)
            .color(MUTED)
            .into(),
    );
    rows.push(
        text("Base height (world Y), fading above.")
            .size(10)
            .color(MUTED)
            .into(),
    );
    rows.push(
        text("IBL ambient: sky/HDRI radiance added to fog.")
            .size(10)
            .color(MUTED)
            .into(),
    );
    rows.push(slider_row(
        "Base density",
        0.0..=0.05,
        volumetrics.density,
        DebugMessage::SetVolumetricDensity,
    ));
    rows.push(slider_row(
        "Base height (Y)",
        -10.0..=20.0,
        volumetrics.height_base,
        DebugMessage::SetVolumetricHeightBase,
    ));
    rows.push(slider_row(
        "Height density",
        0.0..=0.1,
        volumetrics.height_density,
        DebugMessage::SetVolumetricHeightDensity,
    ));
    rows.push(slider_row(
        "Height falloff",
        0.01..=2.0,
        volumetrics.height_falloff,
        DebugMessage::SetVolumetricHeightFalloff,
    ));
    rows.push(slider_row(
        "Scattering",
        0.0..=3.0,
        volumetrics.scattering,
        DebugMessage::SetVolumetricScattering,
    ));
    rows.push(slider_row(
        "Extinction",
        0.0..=3.0,
        volumetrics.extinction,
        DebugMessage::SetVolumetricExtinction,
    ));
    rows.push(slider_row(
        "IBL ambient",
        0.0..=3.0,
        volumetrics.environment_intensity,
        DebugMessage::SetVolumetricEnvironmentIntensity,
    ));
    rows.push(slider_row(
        "Anisotropy",
        -0.95..=0.95,
        volumetrics.anisotropy,
        DebugMessage::SetVolumetricAnisotropy,
    ));
    rows.push(
        text("0 = isotropic, >0 forward-scattering, <0 back-scattering.")
            .size(10)
            .color(MUTED)
            .into(),
    );
    rows.push(slider_row(
        "Max distance",
        10.0..=500.0,
        volumetrics.max_distance,
        DebugMessage::SetVolumetricMaxDistance,
    ));

    rows.push(crate::panels::caption("Froxel quality", MUTED));
    rows.push(
        text("Froxels are view-frustum voxels.")
            .size(10)
            .color(MUTED)
            .into(),
    );
    rows.push(
        text("Higher quality resolves finer light shafts.")
            .size(10)
            .color(MUTED)
            .into(),
    );
    rows.push(
        row![
            radio(
                "Low",
                VolumetricQuality::Low,
                Some(volumetrics.quality),
                DebugMessage::SetVolumetricQuality,
            )
            .text_size(12),
            radio(
                "Medium",
                VolumetricQuality::Medium,
                Some(volumetrics.quality),
                DebugMessage::SetVolumetricQuality,
            )
            .text_size(12),
            radio(
                "High",
                VolumetricQuality::High,
                Some(volumetrics.quality),
                DebugMessage::SetVolumetricQuality,
            )
            .text_size(12),
            radio(
                "Mobile",
                VolumetricQuality::Mobile,
                Some(volumetrics.quality),
                DebugMessage::SetVolumetricQuality,
            )
            .text_size(12),
        ]
        .spacing(6)
        .into(),
    );

    if !volumetrics.enabled {
        rows.push(
            text("Enable volumetrics to see fog and light scattering.")
                .size(11)
                .color(MUTED)
                .into(),
        );
    }

    rows.push(crate::panels::divider());
    rows.push(crate::panels::caption("Debug overlays", ACCENT));
    match ecs.get_resource::<DebugToggleState>() {
        Some(toggle) => {
            let settings = toggle.settings;
            rows.push(
                checkbox(settings.enabled)
                    .label("Enable debug overlay (F3)")
                    .text_size(12)
                    .on_toggle(DebugMessage::SetDebugOverlay)
                    .into(),
            );
            rows.push(
                text(format!("{:?} also toggles this overlay.", toggle.key))
                    .size(10)
                    .color(MUTED)
                    .into(),
            );

            if settings.enabled {
                rows.push(crate::panels::caption("Bounds", MUTED));
                rows.push(
                    row![
                        radio(
                            "Off",
                            BoundsMode::Off,
                            Some(settings.bounds),
                            DebugMessage::SetDebugBounds,
                        )
                        .text_size(11),
                        radio(
                            "Spheres",
                            BoundsMode::Spheres,
                            Some(settings.bounds),
                            DebugMessage::SetDebugBounds,
                        )
                        .text_size(11),
                    ]
                    .spacing(6)
                    .into(),
                );
                rows.push(
                    row![
                        radio(
                            "Boxes",
                            BoundsMode::Boxes,
                            Some(settings.bounds),
                            DebugMessage::SetDebugBounds,
                        )
                        .text_size(11),
                        radio(
                            "Both",
                            BoundsMode::Both,
                            Some(settings.bounds),
                            DebugMessage::SetDebugBounds,
                        )
                        .text_size(11),
                    ]
                    .spacing(6)
                    .into(),
                );
                rows.push(
                    checkbox(settings.frustum)
                        .label("Camera / culling frustum")
                        .text_size(12)
                        .on_toggle(DebugMessage::SetDebugFrustum)
                        .into(),
                );
                rows.push(
                    checkbox(settings.lights)
                        .label("Light indicators")
                        .text_size(12)
                        .on_toggle(DebugMessage::SetDebugLights)
                        .into(),
                );
                rows.push(
                    checkbox(settings.wireframe)
                        .label("Model wireframe")
                        .text_size(12)
                        .on_toggle(DebugMessage::SetDebugWireframe)
                        .into(),
                );
                rows.push(
                    checkbox(settings.grid_axes)
                        .label("World grid & axes")
                        .text_size(12)
                        .on_toggle(DebugMessage::SetDebugGridAxes)
                        .into(),
                );
                rows.push(
                    checkbox(settings.shadow_frusta)
                        .label("Shadow frusta")
                        .text_size(12)
                        .on_toggle(DebugMessage::SetDebugShadowFrusta)
                        .into(),
                );
                rows.push(
                    checkbox(settings.selection)
                        .label("Selected entity highlight")
                        .text_size(12)
                        .on_toggle(DebugMessage::SetDebugSelection)
                        .into(),
                );
                rows.push(
                    text("Wireframe requires GPU line polygon mode.")
                        .size(10)
                        .color(MUTED)
                        .into(),
                );
            } else {
                rows.push(
                    text("Enable the overlay to configure debug categories.")
                        .size(10)
                        .color(MUTED)
                        .into(),
                );
            }
        }
        None => {
            rows.push(
                text("DebugModule is not installed.")
                    .size(11)
                    .color(MUTED)
                    .into(),
            );
        }
    }

    container(crate::panels::scrolling(crate::panels::column_of(
        rows, 5.0,
    )))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn slider_row(
    label: &'static str,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    message: fn(f32) -> DebugMessage,
) -> Element<'static, DebugMessage, Theme, IcedRenderer> {
    column![
        row![
            text(label).size(12).width(Length::Fixed(LABEL_WIDTH)),
            text(format!("{value:.3}"))
                .size(11)
                .color(MUTED)
                .width(Length::Fixed(VALUE_WIDTH)),
        ],
        slider(range, value, message).step(0.001),
    ]
    .spacing(1)
    .into()
}

fn active_projection(ecs: &World) -> Option<ProjectionMode> {
    let active = ecs.get_resource::<ActiveCamera>()?.0;
    ecs.component_debug(TypeId::of::<CameraDescriptor>(), &active)
        .and_then(|handle| {
            handle
                .as_any()
                .downcast_ref::<CameraDescriptor>()
                .map(|camera| camera.projection_mode)
        })
}
