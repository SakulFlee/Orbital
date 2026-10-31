//! Panel exposing live renderer and volumetrics controls.

use std::any::TypeId;

use iced_widget::{checkbox, column, container, pick_list, radio, row, slider, text};
use iced_winit::core::{Color, Element, Length, Theme};

use orbital_app::RenderSettings;
use orbital_camera::ProjectionMode;
use orbital_ecs::World;
use orbital_ecs_bridge::{ActiveCamera, CameraDescriptor, VolumetricQuality, VolumetricSettings};
use orbital_renderer::{AntiAliasing, SmaaQuality};

use crate::state::Renderer as IcedRenderer;
use crate::state::{DebugMessage, DebugUiState, PanelId};

const LABEL_WIDTH: f32 = 108.0;
const VALUE_WIDTH: f32 = 46.0;
const ACCENT: Color = Color::from_rgb(0.7, 0.8, 1.0);
const MUTED: Color = Color::from_rgb(0.6, 0.6, 0.66);

/// Builds the rendering controls panel.
pub fn rendering_panel<'a>(ecs: &World) -> Element<'a, DebugMessage, Theme, IcedRenderer> {
    let Some(state) = ecs.get_resource::<DebugUiState>() else {
        return crate::panels::blank();
    };

    if !state.is_panel_visible(PanelId::Rendering) {
        return crate::panels::blank();
    }

    let render = ecs
        .get_resource::<RenderSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    let volumetrics = ecs
        .get_resource::<VolumetricSettings>()
        .map(|settings| *settings)
        .unwrap_or_default();
    let projection = active_projection(ecs);

    let mut rows: Vec<Element<'static, DebugMessage, Theme, IcedRenderer>> = vec![
        crate::panels::caption("Anti-aliasing", ACCENT),
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
    ];

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
    rows.push(crate::panels::caption("Exposure & bloom", ACCENT));
    rows.push(slider_row(
        "Exposure",
        0.1..=4.0,
        render.exposure,
        DebugMessage::SetExposure,
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
